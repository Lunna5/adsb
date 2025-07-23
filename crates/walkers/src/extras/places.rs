use crate::{MapMemory, Plugin, Position, Projector};
use egui::{vec2, Id, Rect, Response, Sense, Ui};
use log::info;
//use crate::map::ParallelPlugin;

/// [`Plugin`] which shows places on the map. Place can be any type that implements the [`Place`]
/// trait.
pub struct Places<T>
where
    T: Place + Send + Sync + 'static,
{
    places: Vec<T>,
}

impl<T> Places<T>
where
    T: Place + Send + Sync + 'static,
{
    pub fn new(places: Vec<T>) -> Self {
        Self { places }
    }
}

impl<T> Plugin for Places<T>
where
    T: Place + Send + Sync + 'static,
{
    fn run(
        &mut self,
        ui: &mut Ui,
        _response: &Response,
        projector: &Projector,
        _map_memory: &MapMemory,
    ) {
        for place in &self.places {
            place.draw(ui, projector);
        }
    }
}

pub trait Place {
    fn position(&self) -> Position;
    fn draw(&self, ui: &Ui, projector: &Projector);
}

/// A group of places that can be drawn together on the map.
pub trait Group {
    fn draw<T: Place>(&self, places: &[&T], position: Position, projector: &Projector, ui: &mut Ui);
}

/// Similar to [`Places`], but groups places that are close together and draws them as a
/// single [`Group`].
#[derive(Clone, Debug)]
pub struct GroupedPlaces<T, G>
where
    T: Place + Send + Sync + 'static,
    G: Group + Send + Sync + 'static,
{
    places:  Vec<T>,
    group: G,
    current_groups_indexes: Vec<Vec<usize>>,
}

impl<T, G> GroupedPlaces<T, G>
where
    T: Place + Send + Sync + 'static,
    G: Group + Send + Sync + 'static,
{
    pub fn new(places: Vec<T>, group: G) -> Self {
        Self { places, group, current_groups_indexes: Vec::new() }
    }

    /// Handle user interactions. Returns whether group should be expanded.
    fn interact(&self, position: Position, projector: &Projector, ui: &Ui, id: Id) -> bool {
        let screen_position = projector.project(position);
        let rect = Rect::from_center_size(screen_position.to_pos2(), vec2(50., 50.));
        let response = ui.interact(rect, id, Sense::click());

        if response.clicked() {
            // Toggle the visibility of the group when clicked.
            let expand = ui.ctx().memory_mut(|m| {
                let expand = m.data.get_temp::<bool>(id).unwrap_or(false);
                m.data.insert_temp(id, !expand);
                expand
            });
            expand
        } else {
            ui.ctx()
                .memory(|m| m.data.get_temp::<bool>(id).unwrap_or(false))
        }
    }
}

impl<T, G> Plugin for GroupedPlaces<T, G>
where
    T: Place + Send + Sync + 'static,
    G: Group + Send + Sync + 'static,
{
    fn run(
        &mut self,
        ui: &mut Ui,
        _response: &Response,
        projector: &Projector,
        _map_memory: &MapMemory,
    ) {
        for (idx, group) in self.current_groups_indexes.iter().enumerate() {
            let id = ui.id().with(idx);
            let positions: Vec<_> = group.iter().map(|&i| self.places[i].position()).collect();
            let position = center(&positions);
            let expand = self.interact(position, projector, ui, id);

            if group.len() >= 2 && !expand {
                let refs: Vec<&T> = group.iter().map(|&i| &self.places[i]).collect();
                self.group.draw(&refs, position, projector, ui);
            } else {
                for &i in group {
                    self.places[i].draw(ui, projector);
                }
            }
        }
    }

    fn is_parallel(&self) -> bool {
        true
    }

    fn parallel_run(&mut self, projector: &Projector, map_memory: &MapMemory) {
        profiling::scope!("GroupedPlaces::parallel_run");
        let groups = self.groups(projector);
        self.current_groups_indexes = groups;
    }
}

// impl<T, G> ParallelPlugin for GroupedPlaces<T, G>
// where
//     T: Place + Send + Sync + 'static,
//     G: Group + Send + Sync + 'static,
// {

// }



impl <T, G> GroupedPlaces<T, G>
where
    T: Place + Send + Sync + 'static,
    G: Group + Send + Sync + 'static,
{
    /// Group places that are close together.
    /// TODO: Delegate this function to the GPU
    pub fn groups(&self, projector: &Projector) -> Vec<Vec<usize>> {
        let mut groups: Vec<Vec<usize>> = Vec::new();
        let zoom: f64 = projector.memory.zoom.into();

        if self.places.len() == 1 {
            return if zoom < 6.0 {
                vec![]
            } else {
                vec![vec![0]]
            }
        }

        for (idx, place) in self.places.iter().enumerate() {
            let place_position = place.position();

            if !projector.is_in_view(place_position) {
                continue;
            }

            if zoom < 6.0 {
                if groups.is_empty() {
                    groups.push(vec![idx]);
                } else {
                    groups[0].push(idx);
                }
                continue;
            }

            if let Some(group) = groups.iter_mut().find(|g| {
                g.iter().all(|&i| {
                    distance_projected(place_position, self.places[i].position(), projector) < 50.0
                })
            }) {
                group.push(idx);
            } else {
                groups.push(vec![idx]);
            }
        }

        groups
    }
}

/// Calculate the distance between two positions after being projected onto the screen.
fn distance_projected(p1: Position, p2: Position, projector: &Projector) -> f32 {
    let screen_p1 = projector.project(p1).to_pos2();
    let screen_p2 = projector.project(p2).to_pos2();
    (screen_p1 - screen_p2).length()
}

fn center(positions: &[Position]) -> Position {
    if positions.is_empty() {
        Position::default()
    } else {
        let sum = positions
            .iter()
            .fold(Position::default(), |acc, &p| acc + p);
        sum / positions.len() as f64
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn calculating_center() {
        use super::*;

        assert_eq!(
            center(&[
                Position::new(0.0, 0.0),
                Position::new(10.0, 10.0),
                Position::new(20.0, 20.0),
            ]),
            Position::new(10.0, 10.0)
        );

        assert_eq!(
            center(&[
                Position::new(0.0, 0.0),
                Position::new(10.0, 0.0),
                Position::new(0.0, 10.0),
                Position::new(10.0, 10.0),
            ]),
            Position::new(5.0, 5.0)
        );

        assert_eq!(
            center(&[
                Position::new(10.0, 10.0),
                Position::new(-10.0, -10.0),
                Position::new(-10.0, 10.0),
                Position::new(10.0, -10.0),
            ]),
            Position::new(0.0, 0.0)
        );
    }
}
