use std::collections::VecDeque;

use egui::{Image, Pos2, Rect, Vec2};
use hashbrown::{HashMap, HashSet};
use spice_rs::{
    elements::{
        capacitor::Capacitor, dc_current_source::DCCurrentSource,
        dc_voltage_source::DCVoltageSource, inductor::Inductor, resistor::Resistor,
    },
    Circuit, NodeId,
};

use crate::{
    utils::{
        double_range::{DoubleRange, DoubleRangeInclusive},
        ipos2::{IPos2, Pos2Ext},
        vec2ext::Vec2Ext,
    },
    GRID_SIZE,
};

#[derive(Default)]
pub struct GuiCircuit {
    /// List of wire points and its neighbors
    pub nodes: HashMap<IPos2, Vec<IPos2>>,
    /// List of starting positions of a node
    pub nodes_starts: Vec<IPos2>,
    /// Groups of wires into nodes
    pub node_groups: Vec<HashSet<IPos2>>,
    /// Wire points to be rendered
    pub rendered_wires: Vec<Vec<Pos2>>,
    /// Circuit elements that are currently placed
    pub gui_elements: HashMap<u32, GuiElement>,
    /// Element ids that were used but are currently free to allocate
    free_ids: Vec<u32>,
}

impl GuiCircuit {
    pub fn construct_circuit(&self) -> Option<Circuit> {
        let mut circuit = Circuit::default();

        let Some((_, ground)) = self
            .gui_elements
            .iter()
            .find(|x| x.1.element == ElementType::Ground)
        else {
            return None;
        };
        let ground_node = self
            .node_groups
            .iter()
            .enumerate()
            .find(|x| x.1.contains(&ground.nodes[0]))
            .unwrap()
            .0;
        println!("Ground {}", ground_node);

        let nodes = self
            .node_groups
            .iter()
            .enumerate()
            .map(|x| NodeId(x.0))
            .collect::<Vec<NodeId>>();
        println!("{:?}", self.node_groups);
        println!("Nodes {} {:?}", nodes.len(), nodes);
        circuit.nodes = nodes;

        let mut bc_amount = 0;
        for (_, gui_element) in self.gui_elements.iter() {
            if gui_element.element == ElementType::Ground {
                continue;
            }

            let Some(node_group1) = self
                .node_groups
                .iter()
                .enumerate()
                .find(|x| x.1.contains(&gui_element.nodes[0]))
            else {
                continue;
            };
            let node1 = Self::transcribe_node(node_group1.0, ground_node);

            let Some(node_group2) = self
                .node_groups
                .iter()
                .enumerate()
                .find(|x| x.1.contains(&gui_element.nodes[1]))
            else {
                continue;
            };
            let node2 = Self::transcribe_node(node_group2.0, ground_node);

            match gui_element.element {
                ElementType::Resistor(resistance) => {
                    circuit.add_element(Box::new(Resistor::new(resistance, node1, node2)));
                    println!("Add resistor ({}, {})", node1.0, node2.0);
                }
                ElementType::DCVoltageSource(voltage) => {
                    circuit.add_element(Box::new(DCVoltageSource::new(
                        voltage, node1, node2, bc_amount,
                    )));
                    println!("Add DC Voltage Source ({}, {})", node1.0, node2.0);

                    bc_amount += 1;
                }
                ElementType::DCCurrentSource(amps) => {
                    circuit.add_element(Box::new(DCCurrentSource::new(amps, node1, node2)));
                    println!("Add DC Current Source ({}, {})", node1.0, node2.0);
                }
                ElementType::Capacitor(capacitance) => {
                    circuit.add_element(Box::new(Capacitor::new(capacitance, node1, node2)));
                    println!("Add Capacitor ({}, {})", node1.0, node2.0);
                }
                ElementType::Inductor(capacitance) => {
                    circuit.add_element(Box::new(Inductor::new(
                        capacitance,
                        node1,
                        node2,
                        bc_amount,
                    )));
                    println!("Add Inductor ({}, {})", node1.0, node2.0);

                    bc_amount += 1;
                }
                ElementType::Ground => (),
            }
        }

        Some(circuit)
    }

    fn transcribe_node(node: usize, ground_node: usize) -> NodeId {
        if ground_node != 0 {
            if node == 0 {
                return NodeId(ground_node);
            } else if node == ground_node {
                return NodeId(0);
            }
        }

        NodeId(node)
    }

    pub fn add_element(&mut self, mut element: GuiElement) {
        let id = self
            .free_ids
            .pop()
            .unwrap_or(self.gui_elements.len() as u32);
        element.id = id;

        self.gui_elements.insert(element.id, element);
    }

    pub fn remove_element(&mut self, id: u32) {
        if let Some(element) = self.gui_elements.remove(&id) {
            self.free_ids.push(id);

            for node_position in element.nodes.into_iter() {
                let indexes = self
                    .nodes_starts
                    .iter()
                    .enumerate()
                    .filter(|x| *x.1 == node_position)
                    .map(|x| x.0)
                    .collect::<Vec<usize>>();
                for index in indexes {
                    self.nodes_starts.remove(index);
                }
            }
        }
    }

    pub fn remove_node(&mut self, position: IPos2) {
        let Some((group_index, _)) = self
            .node_groups
            .iter()
            .enumerate()
            .find(|x| x.1.contains(&position))
        else {
            println!("Return");
            return;
        };
        println!("Remove Group {}", group_index);
        let group = self.node_groups.swap_remove(group_index);

        group.iter().for_each(|position| {
            self.nodes.remove(position);
        });
        if let Some(starting_point) = self.nodes_starts.iter().find(|x| group.contains(*x)) {
            self.rendered_wires
                .retain(|x| !x.contains(&starting_point.to_pos2()));
        }
        self.nodes_starts.retain(|x| !group.contains(x));
    }

    pub fn add_orthogonal_wires(&mut self, start: IPos2, end: IPos2, x_first: bool) {
        if !self.nodes_starts.contains(&end) {
            self.nodes_starts.push(end);
        }
        if !self.nodes_starts.contains(&start) {
            self.nodes_starts.push(start);
        }

        let middle_position = match x_first {
            true => IPos2::new(end.x, start.y),
            false => IPos2::new(start.x, end.y),
        };
        let mut previous_position = None;

        self.rendered_wires.push(vec![
            start.to_pos2(),
            middle_position.to_pos2(),
            end.to_pos2(),
        ]);

        if x_first {
            self.add_wire_axis(
                &mut previous_position,
                middle_position,
                DoubleRange::new(start.x, middle_position.x, 1),
                true,
            );
            self.add_wire_axis(
                &mut previous_position,
                middle_position,
                DoubleRangeInclusive::new(middle_position.y, end.y, 1),
                false,
            );
        } else {
            self.add_wire_axis(
                &mut previous_position,
                middle_position,
                DoubleRange::new(start.y, middle_position.y, 1),
                false,
            );
            self.add_wire_axis(
                &mut previous_position,
                middle_position,
                DoubleRangeInclusive::new(middle_position.x, end.x, 1),
                true,
            );
        }

        self.group_wires_into_nodes();
    }

    fn add_wire_axis(
        &mut self,
        previous_position: &mut Option<IPos2>,
        middle_position: IPos2,
        range: impl Iterator<Item = i32>,
        x_axis: bool,
    ) {
        for i in range {
            let position = match x_axis {
                true => IPos2::new(i, middle_position.y),
                false => IPos2::new(middle_position.x, i),
            };

            if let Some(previous_position) = previous_position {
                if let Some(neighbors) = self.nodes.get_mut(previous_position) {
                    if !neighbors.contains(&position) {
                        neighbors.push(position);
                    }
                }
            }

            if let Some(neighbors) = self.nodes.get_mut(&position) {
                if let Some(previous_position) = previous_position {
                    if !neighbors.contains(&previous_position) {
                        neighbors.push(*previous_position);
                    }
                }
            } else {
                self.nodes.insert(
                    position,
                    previous_position.iter().map(|x| *x).collect::<Vec<IPos2>>(),
                );
            }

            *previous_position = Some(position);
        }
    }

    fn group_wires_into_nodes(&mut self) {
        let mut wires = self.nodes.clone();
        let mut wire_starts = self.nodes_starts.clone();
        let mut visited: HashSet<IPos2> = HashSet::new();
        let mut queue: VecDeque<IPos2> = VecDeque::new();
        let mut group: HashSet<IPos2> = HashSet::new();
        let mut groups: Vec<HashSet<IPos2>> = vec![];

        let Some(start) = wire_starts.pop() else {
            return;
        };
        wires.remove(&start);
        wire_starts.swap_remove(0);
        visited.insert(start);
        queue.push_back(start);
        group.insert(start);

        while !wires.is_empty() {
            let position = queue.pop_front().unwrap();
            let neighbors = self.nodes.get(&position).unwrap();

            visited.insert(position);

            for neighbor in neighbors.iter() {
                if visited.contains(neighbor) {
                    continue;
                }

                group.insert(*neighbor);
                queue.push_back(*neighbor);
                wires.remove(neighbor);
                if wire_starts.contains(neighbor) {
                    let index = wire_starts
                        .iter()
                        .enumerate()
                        .find(|x| x.1 == neighbor)
                        .map(|x| x.0)
                        .unwrap();
                    wire_starts.remove(index);
                }
            }

            if queue.is_empty() || wires.is_empty() {
                groups.push(std::mem::take(&mut group));
                if let Some(wire_start) = wire_starts.pop() {
                    queue.push_back(wire_start);
                    wires.remove(&wire_start);
                    group.insert(wire_start);
                }
            }
        }

        self.node_groups = groups;
    }
}

#[derive(Clone)]
pub struct GuiElement {
    id: u32,
    pub element: ElementType,
    pub rect: Rect,
    pub nodes: Vec<IPos2>,
    pub image: Image<'static>,
}

impl GuiElement {
    pub fn new(element: ElementType, rect: Rect, image: Image<'static>) -> Self {
        let rotation = image.image_options().rotation.unwrap_or_default().0.angle();
        Self {
            id: 0,
            element,
            rect,
            nodes: element.node_positions(rect.center(), rotation),
            image: image.rotate(rotation, Vec2::splat(0.5)),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ElementType {
    Ground,
    Resistor(f32),
    DCVoltageSource(f32),
    DCCurrentSource(f32),
    Capacitor(f32),
    Inductor(f32),
}

impl ElementType {
    fn node_positions(&self, center: Pos2, rotation: f32) -> Vec<IPos2> {
        match self {
            ElementType::Ground => {
                vec![(center - Vec2::new(0.0, 16.0).rotate(rotation)).to_ipos2(GRID_SIZE)]
            }
            _ => vec![
                (center - Vec2::new(32.0, 0.0).rotate(rotation)).to_ipos2(GRID_SIZE),
                (center + Vec2::new(32.0, 0.0).rotate(rotation)).to_ipos2(GRID_SIZE),
            ],
        }
    }

    pub fn value_mut(&mut self) -> Option<&mut f32> {
        match self {
            ElementType::Resistor(resistance) => Some(resistance),
            ElementType::DCVoltageSource(voltage) => Some(voltage),
            ElementType::DCCurrentSource(amps) => Some(amps),
            ElementType::Capacitor(capacitance) => Some(capacitance),
            ElementType::Inductor(inductance) => Some(inductance),
            ElementType::Ground => None,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            ElementType::Ground => "Ground",
            ElementType::Resistor(_) => "Resistor",
            ElementType::DCVoltageSource(_) => "DC Voltage Source",
            ElementType::DCCurrentSource(_) => "DC Current Source",
            ElementType::Capacitor(_) => "Capacitor",
            ElementType::Inductor(_) => "Inductor",
        }
    }

    pub fn display_unit_name(&self) -> Option<&'static str> {
        match self {
            ElementType::Resistor(_) => Some("Resistance"),
            ElementType::DCVoltageSource(_) => Some("Voltage"),
            ElementType::DCCurrentSource(_) => Some("Current"),
            ElementType::Capacitor(_) => Some("Capacitance"),
            ElementType::Inductor(_) => Some("Inductance"),
            _ => None,
        }
    }

    pub fn display_unit_symbol(&self) -> Option<&'static str> {
        match self {
            ElementType::Resistor(_) => Some("Ω"),
            ElementType::DCVoltageSource(_) => Some("V"),
            ElementType::DCCurrentSource(_) => Some("A"),
            ElementType::Capacitor(_) => Some("F"),
            ElementType::Inductor(_) => Some("H"),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct ToPlaceElement {
    pub element: ElementType,
    pub image: Image<'static>,
}

impl ToPlaceElement {
    pub fn new(element: ElementType, image: Image<'static>) -> Self {
        Self { element, image }
    }
}

#[cfg(test)]
mod tests {
    use egui::Pos2;
    use hashbrown::{HashMap, HashSet};

    use crate::utils::ipos2::IPos2;

    use super::GuiCircuit;

    #[test]
    fn add_orthogonal_wires_x_first() {
        let mut circuit = GuiCircuit::default();
        circuit.add_orthogonal_wires(IPos2::new(0, 0), IPos2::new(3, 3), true);

        assert_eq!(circuit.nodes_starts, [IPos2::new(3, 3), IPos2::new(0, 0)]);

        let mut test_nodes = HashMap::new();
        test_nodes.insert(IPos2::new(0, 0), vec![IPos2::new(1, 0)]);
        test_nodes.insert(IPos2::new(1, 0), vec![IPos2::new(0, 0), IPos2::new(2, 0)]);
        test_nodes.insert(IPos2::new(2, 0), vec![IPos2::new(1, 0), IPos2::new(3, 0)]);
        test_nodes.insert(IPos2::new(3, 0), vec![IPos2::new(2, 0), IPos2::new(3, 1)]);
        test_nodes.insert(IPos2::new(3, 1), vec![IPos2::new(3, 0), IPos2::new(3, 2)]);
        test_nodes.insert(IPos2::new(3, 2), vec![IPos2::new(3, 1), IPos2::new(3, 3)]);
        test_nodes.insert(IPos2::new(3, 3), vec![IPos2::new(3, 2)]);

        assert_eq!(circuit.nodes, test_nodes);

        let test_groups = HashSet::from_iter(
            vec![
                IPos2::new(0, 0),
                IPos2::new(1, 0),
                IPos2::new(2, 0),
                IPos2::new(3, 0),
                IPos2::new(3, 1),
                IPos2::new(3, 2),
                IPos2::new(3, 3),
            ]
            .into_iter(),
        );
        assert_eq!(circuit.node_groups, vec![test_groups]);
    }

    #[test]
    fn add_orthogonal_wires_y_first() {
        let mut circuit = GuiCircuit::default();
        circuit.add_orthogonal_wires(IPos2::new(0, 0), IPos2::new(3, 3), false);

        assert_eq!(circuit.nodes_starts, [IPos2::new(3, 3), IPos2::new(0, 0)]);

        let mut test_nodes = HashMap::new();
        test_nodes.insert(IPos2::new(0, 0), vec![IPos2::new(0, 1)]);
        test_nodes.insert(IPos2::new(0, 1), vec![IPos2::new(0, 0), IPos2::new(0, 2)]);
        test_nodes.insert(IPos2::new(0, 2), vec![IPos2::new(0, 1), IPos2::new(0, 3)]);
        test_nodes.insert(IPos2::new(0, 3), vec![IPos2::new(0, 2), IPos2::new(1, 3)]);
        test_nodes.insert(IPos2::new(1, 3), vec![IPos2::new(0, 3), IPos2::new(2, 3)]);
        test_nodes.insert(IPos2::new(2, 3), vec![IPos2::new(1, 3), IPos2::new(3, 3)]);
        test_nodes.insert(IPos2::new(3, 3), vec![IPos2::new(2, 3)]);

        assert_eq!(circuit.nodes, test_nodes);

        let test_groups = HashSet::from_iter(
            vec![
                IPos2::new(0, 0),
                IPos2::new(0, 1),
                IPos2::new(0, 2),
                IPos2::new(0, 3),
                IPos2::new(1, 3),
                IPos2::new(2, 3),
                IPos2::new(3, 3),
            ]
            .into_iter(),
        );
        assert_eq!(circuit.node_groups, vec![test_groups]);
    }

    #[test]
    fn reverse_wires() {
        let mut circuit = GuiCircuit::default();
        circuit.add_orthogonal_wires(IPos2::new(2, 1), IPos2::new(0, 0), true);

        assert_eq!(circuit.nodes_starts, [IPos2::new(0, 0), IPos2::new(2, 1)]);

        let mut test_nodes = HashMap::new();
        test_nodes.insert(IPos2::new(2, 1), vec![IPos2::new(1, 1)]);
        test_nodes.insert(IPos2::new(1, 1), vec![IPos2::new(2, 1), IPos2::new(0, 1)]);
        test_nodes.insert(IPos2::new(0, 1), vec![IPos2::new(1, 1), IPos2::new(0, 0)]);
        test_nodes.insert(IPos2::new(0, 0), vec![IPos2::new(0, 1)]);

        assert_eq!(circuit.nodes, test_nodes);

        let test_groups = HashSet::from_iter(
            vec![
                IPos2::new(2, 1),
                IPos2::new(1, 1),
                IPos2::new(0, 1),
                IPos2::new(0, 0),
            ]
            .into_iter(),
        );
        assert_eq!(circuit.node_groups, vec![test_groups]);
    }

    #[test]
    fn two_connected_wires() {
        let mut circuit = GuiCircuit::default();
        circuit.add_orthogonal_wires(IPos2::new(0, 0), IPos2::new(2, 2), true);
        circuit.add_orthogonal_wires(IPos2::new(0, 0), IPos2::new(2, 2), false);

        let mut test_nodes = HashMap::new();
        // X
        test_nodes.insert(IPos2::new(0, 0), vec![IPos2::new(1, 0), IPos2::new(0, 1)]);
        test_nodes.insert(IPos2::new(1, 0), vec![IPos2::new(0, 0), IPos2::new(2, 0)]);
        test_nodes.insert(IPos2::new(2, 0), vec![IPos2::new(1, 0), IPos2::new(2, 1)]);
        test_nodes.insert(IPos2::new(2, 1), vec![IPos2::new(2, 0), IPos2::new(2, 2)]);
        test_nodes.insert(IPos2::new(2, 2), vec![IPos2::new(2, 1), IPos2::new(1, 2)]);

        // Y
        test_nodes.insert(IPos2::new(0, 1), vec![IPos2::new(0, 0), IPos2::new(0, 2)]);
        test_nodes.insert(IPos2::new(0, 2), vec![IPos2::new(0, 1), IPos2::new(1, 2)]);
        test_nodes.insert(IPos2::new(1, 2), vec![IPos2::new(0, 2), IPos2::new(2, 2)]);

        assert_eq!(circuit.nodes, test_nodes);

        let test_groups = HashSet::from_iter(
            vec![
                IPos2::new(0, 0),
                IPos2::new(1, 0),
                IPos2::new(0, 1),
                IPos2::new(2, 0),
                IPos2::new(0, 2),
                IPos2::new(2, 1),
                IPos2::new(1, 2),
                IPos2::new(2, 2),
            ]
            .into_iter(),
        );
        assert_eq!(circuit.node_groups, vec![test_groups]);
    }

    #[test]
    fn overlapping_wires() {
        let mut circuit = GuiCircuit::default();
        circuit.add_orthogonal_wires(IPos2::new(0, 0), IPos2::new(1, 0), true);
        circuit.add_orthogonal_wires(IPos2::new(0, 0), IPos2::new(2, 0), true);

        let mut test_nodes = HashMap::new();
        test_nodes.insert(IPos2::new(0, 0), vec![IPos2::new(1, 0)]);
        test_nodes.insert(IPos2::new(1, 0), vec![IPos2::new(0, 0), IPos2::new(2, 0)]);
        test_nodes.insert(IPos2::new(2, 0), vec![IPos2::new(1, 0)]);

        assert_eq!(circuit.nodes, test_nodes);

        let test_groups = HashSet::from_iter(
            vec![IPos2::new(0, 0), IPos2::new(1, 0), IPos2::new(2, 0)].into_iter(),
        );
        assert_eq!(circuit.node_groups, vec![test_groups]);
    }

    #[test]
    fn two_groups() {
        let mut circuit = GuiCircuit::default();
        circuit.add_orthogonal_wires(IPos2::new(0, 0), IPos2::new(1, 0), true);
        circuit.add_orthogonal_wires(IPos2::new(0, 1), IPos2::new(1, 1), true);

        let mut test_nodes = HashMap::new();
        test_nodes.insert(IPos2::new(0, 0), vec![IPos2::new(1, 0)]);
        test_nodes.insert(IPos2::new(1, 0), vec![IPos2::new(0, 0)]);
        test_nodes.insert(IPos2::new(0, 1), vec![IPos2::new(1, 1)]);
        test_nodes.insert(IPos2::new(1, 1), vec![IPos2::new(0, 1)]);

        assert_eq!(circuit.nodes, test_nodes);

        let test_group1 = HashSet::from_iter(vec![IPos2::new(0, 1), IPos2::new(1, 1)].into_iter());
        let test_group2 = HashSet::from_iter(vec![IPos2::new(0, 0), IPos2::new(1, 0)].into_iter());
        assert_eq!(circuit.node_groups, vec![test_group1, test_group2]);
    }

    #[test]
    fn remove_wires() {
        let mut circuit = GuiCircuit::default();
        circuit.add_orthogonal_wires(IPos2::new(0, 0), IPos2::new(10, 0), true);

        circuit.remove_node(IPos2::new(0, 0));

        assert_eq!(circuit.nodes, HashMap::new());
        assert_eq!(circuit.node_groups, vec![]);
        assert_eq!(circuit.nodes_starts, vec![]);
        assert_eq!(circuit.rendered_wires, Vec::<Vec<Pos2>>::new());
    }
}
