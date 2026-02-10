/// Route optimization flow assignment problem
/// **IMPORTANT:** This assumes nodes have continuous, gapless ids starting from 0.
use rand::prelude::*;
use std::{
    any::Any,
    collections::{HashMap, HashSet, VecDeque},
};

use shared::settings::ProblemSettings;

use crate::ga::{get_id, Individual, Problem};

type Id = usize;

#[derive(Debug, Clone)]
struct Node {
    id: Id,
}

impl Node {
    fn with_id(id: Id) -> Self {
        Self { id }
    }

    fn id(&self) -> Id {
        self.id
    }
}

#[derive(Debug, Clone)]
struct BidirectionalLink {
    connected_nodes: (Id, Id),
}

impl BidirectionalLink {
    fn between(connected_nodes: (Id, Id)) -> Self {
        assert!(
            connected_nodes.0 != connected_nodes.1,
            "Cannot connect node to itself"
        );
        Self { connected_nodes }
    }

    fn is_between(&self, nodes: (Id, Id)) -> bool {
        (nodes.0 == self.connected_nodes.0 && nodes.1 == self.connected_nodes.1)
            || (nodes.0 == self.connected_nodes.1 && nodes.1 == self.connected_nodes.0)
    }

    fn froms(&self) -> Vec<Id> {
        vec![self.connected_nodes.0, self.connected_nodes.1]
    }

    fn is_from(&self, from: Id) -> bool {
        self.connected_nodes.0 == from || self.connected_nodes.1 == from
    }

    fn to(&self, from: Id) -> Id {
        match self.connected_nodes {
            (a, b) if a == from => b,
            (a, b) if b == from => a,
            _ => panic!("Node not connected by id"),
        }
    }

    fn random_links(nodes: &Vec<Node>, number_links: usize) -> Vec<Self> {
        let number_nodes = nodes.len();
        let mut rng = rand::rng();
        let mut links = Vec::with_capacity(number_links);
        for _ in 0..number_links {
            let mut between = (
                rng.random_range(0..number_nodes),
                rng.random_range(0..number_nodes),
            );
            while between.0 == between.1
                || links
                    .iter()
                    .any(|l: &BidirectionalLink| l.is_between(between))
            {
                between = (
                    rng.random_range(0..number_nodes),
                    rng.random_range(0..number_nodes),
                );
            }
            links.push(BidirectionalLink::between(between));
        }
        links
    }
}

//TODO: handle other type
struct UnidirectionalLink {}
enum Link {}

//TODO: handle other type
struct UnidirectionalDemand {}
enum Demand {}

const MAX_TRAFFIC_DEMAND: usize = 10;

#[derive(Debug, Clone)]
struct BidirectionalDemand {
    demand_nodes: (Id, Id),
    required_traffic: usize,
}

impl BidirectionalDemand {
    fn between_with_traffic(demand_nodes: (Id, Id), required_traffic: usize) -> Self {
        assert!(
            demand_nodes.0 != demand_nodes.1,
            "Cannot demand traffic from node to itself"
        );
        Self {
            demand_nodes,
            required_traffic,
        }
    }

    fn is_between(&self, nodes: (Id, Id)) -> bool {
        (nodes.0 == self.demand_nodes.0 && nodes.1 == self.demand_nodes.1)
            || (nodes.0 == self.demand_nodes.1 && nodes.1 == self.demand_nodes.0)
    }

    fn random_demands(number_nodes: usize, number_demands: usize) -> Vec<Self> {
        let mut rng = rand::rng();
        let mut demands = Vec::with_capacity(number_demands);
        for _ in 0..number_demands {
            let mut between = (
                rng.random_range(0..number_nodes),
                rng.random_range(0..number_nodes),
            );
            //TODO: this is a bad idea, edge cases can take a long time
            while between.0 == between.1
                || demands
                    .iter()
                    .any(|d: &BidirectionalDemand| d.is_between(between))
            {
                between = (
                    rng.random_range(0..number_nodes),
                    rng.random_range(0..number_nodes),
                );
            }
            let required_traffic = rng.random_range(0..=MAX_TRAFFIC_DEMAND);
            demands.push(BidirectionalDemand::between_with_traffic(
                between,
                required_traffic,
            ));
        }
        demands
    }
}

/// Represents a network with nodes and links.
/// # Important
/// It is assumed that the order of nodes and the order of links never changes,
/// as the same order is used in Routing and Capacity Plans.
#[derive(Debug, Clone)]
pub struct Network {
    nodes: Vec<Node>,
    links: Vec<BidirectionalLink>,
    link_map_start_id: HashMap<Id, Vec<BidirectionalLink>>,
    index_map_start_and_end_id: HashMap<(Id, Id), usize>,
}

impl Network {
    fn random(number_nodes: usize, number_links: usize) -> Self {
        assert!(
            number_links >= number_nodes - 1,
            "Not enough links to build a coherent network"
        );
        assert!(
            number_links <= number_nodes * (number_nodes - 1) / 2,
            "Cannot have unique links, there are too many"
        );
        let nodes = (0..number_nodes).map(Node::with_id).collect();
        let mut links = BidirectionalLink::random_links(&nodes, number_links);
        //TODO: this is a bad idea, edge cases can take a long time
        while !Self::coherent(&nodes, &links) {
            links = BidirectionalLink::random_links(&nodes, number_links);
        }
        let link_map_start_id = Self::create_link_map(&links);
        let index_map_start_and_end_id = Self::create_index_map(&links);
        Self {
            nodes,
            links,
            link_map_start_id,
            index_map_start_and_end_id,
        }
    }

    fn from(nodes: Vec<Node>, links: Vec<BidirectionalLink>) -> Self {
        let link_map_start_id = Self::create_link_map(&links);
        let index_map_start_and_end_id = Self::create_index_map(&links);
        Self {
            nodes,
            links,
            link_map_start_id,
            index_map_start_and_end_id,
        }
    }

    pub fn link_index_by_start_and_end_id(&self, start_and_end_id: (Id, Id)) -> usize {
        *self
            .index_map_start_and_end_id
            .get(&start_and_end_id)
            .expect("No fitting index in index map")
    }

    fn create_index_map(links: &Vec<BidirectionalLink>) -> HashMap<(Id, Id), usize> {
        let mut index_map_start_and_end_id = HashMap::new();
        for (i, link) in links.iter().enumerate() {
            for from in link.froms() {
                let to = link.to(from);
                let previous_value = index_map_start_and_end_id.insert((from, to), i);
                assert!(previous_value.is_none(), "Links should be unique");
            }
        }
        index_map_start_and_end_id
    }

    pub fn filter_by_start_id(&self, start_id: &Id) -> Vec<BidirectionalLink> {
        self.link_map_start_id
            .get(start_id)
            .expect("Cannot filter links by start id")
            .clone()
    }

    fn create_link_map(links: &Vec<BidirectionalLink>) -> HashMap<Id, Vec<BidirectionalLink>> {
        let mut link_map = HashMap::new();
        for link in links {
            let froms = link.froms();
            for from in froms {
                link_map
                    .entry(from)
                    .or_insert(Vec::new())
                    .push(link.clone());
            }
        }
        link_map
    }

    /// Checks if a network is coherent.
    /// This is done using Kosaraju's algorithm.
    /// # Arguments
    /// * `nodes` - A vec of nodes.
    /// * `links` - A vec of links.
    /// # Returns
    /// A `bool` true if the network is coherent, false otherwise.
    fn coherent(nodes: &Vec<Node>, links: &Vec<BidirectionalLink>) -> bool {
        let start_node_id = nodes.first().expect("No nodes provided").id();
        let mut visited_nodes_ids = Vec::new();
        let mut connected_nodes_ids = Vec::new();
        connected_nodes_ids.push(start_node_id);
        while let Some(current_node_id) = connected_nodes_ids.pop() {
            for link in links {
                if link.is_from(current_node_id) {
                    let connected_node_id = link.to(current_node_id);
                    if !connected_nodes_ids.contains(&connected_node_id)
                        && !visited_nodes_ids.contains(&connected_node_id)
                    {
                        connected_nodes_ids.push(connected_node_id);
                    }
                }
            }
            visited_nodes_ids.push(current_node_id);
        }
        nodes.len() == visited_nodes_ids.len()
    }

    fn neighbors(&self, from: Id) -> Vec<Id> {
        let link_map_tos = self.filter_by_start_id(&from);
        link_map_tos.iter().map(|t| t.to(from)).collect()
    }
}

#[derive(Debug, Clone)]
struct CapacityType {
    capacity: usize,
    fixed_cost: f64,
    variable_cost: f64,
}

impl CapacityType {
    fn from_percentage(percentage: f64) -> Self {
        let multiplier = (percentage * 10.0) as usize;
        let capacity = 10u32.pow(3 + multiplier as u32) as usize;
        let fixed_cost = 0.05 * multiplier as f64;
        let variable_cost = 1.0 / (multiplier + 1) as f64;
        Self {
            capacity,
            fixed_cost,
            variable_cost,
        }
    }
}

#[derive(Debug, Clone)]
struct CapacityTypes {
    capacity_types: Vec<CapacityType>,
}

impl CapacityTypes {
    fn random(number: usize) -> Self {
        let capacity_types = (1..=number)
            .map(|n| CapacityType::from_percentage(n as f64 / number as f64))
            .collect();
        Self { capacity_types }
    }

    fn max(&self) -> CapacityType {
        let mut max = self
            .capacity_types
            .first()
            .expect("At least one capacity type is required");
        for capacity_type in &self.capacity_types {
            if capacity_type.capacity > max.capacity {
                max = capacity_type;
            }
        }
        max.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Rofa {
    network: Network,
    demands: Vec<BidirectionalDemand>,
    capacity_types: CapacityTypes,
    problem_settings: ProblemSettings,
}

impl Rofa {
    pub fn random(problem_settings: &ProblemSettings) -> Self {
        let (number_nodes, links_percentage, demands_percentage, number_link_types) =
            match problem_settings {
                ProblemSettings::Rofa {
                    nodes,
                    links_percentage,
                    demands_percentage,
                    link_types,
                } => (*nodes, *links_percentage, *demands_percentage, *link_types),
                _ => panic!(
                "Route Optimization Flow Assignment Problem requires different kind of settings"
            ),
            };
        let links_min: usize = number_nodes - 1;
        let links_max: usize = (number_nodes as f64 * (number_nodes as f64 - 1.0) * 0.5) as usize;
        let number_links = ((links_max - links_min) as f64 * links_percentage as f64 * 0.01).ceil()
            as usize
            + links_min;
        let demands_min = 1;
        let demands_max = number_links; // TODO: correct value
        let number_demands = ((demands_max - demands_min) as f64 * demands_percentage as f64 * 0.01)
            .ceil() as usize
            + demands_min;
        let network = Network::random(number_nodes, number_links);
        let demands = BidirectionalDemand::random_demands(number_nodes, number_demands);
        let capacity_types = CapacityTypes::random(number_link_types);
        Self {
            network,
            demands,
            capacity_types,
            problem_settings: problem_settings.clone(),
        }
    }

    fn cost(&self, routing_and_capacity_plan: &RoutingAndCapacityPlan) -> f64 {
        let planned_capacity_types = &routing_and_capacity_plan
            .capacity_plan
            .planned_capacity_types;

        let all_links_demands = routing_and_capacity_plan.get_links_demands();

        let mut unweighted_delay_cost = 0.0;
        let mut fixed_cost = 0.0;
        let mut variable_cost = 0.0;
        for (&total_demand, planned_capacity_type) in
            all_links_demands.iter().zip(planned_capacity_types)
        {
            let cap = planned_capacity_type.capacity;
            assert!(
                planned_capacity_type.capacity > total_demand,
                "{cap} vs {total_demand} Demand cannot be bigger than capacity"
            );
            let delay =
                total_demand as f64 / (planned_capacity_type.capacity - total_demand) as f64;
            unweighted_delay_cost += delay;
            fixed_cost += planned_capacity_type.fixed_cost;
            variable_cost += (total_demand as f64) * planned_capacity_type.variable_cost;
        }
        let delay_cost = DELAY_COST_COEFFICIENT * unweighted_delay_cost;

        delay_cost + fixed_cost + variable_cost
    }
}

impl Problem for Rofa {
    type Individual = RoutingAndCapacityPlan;

    fn random(problem_settings: &ProblemSettings) -> Self {
        Self::random(problem_settings)
    }

    fn random_individual(&self) -> Self::Individual {
        RoutingAndCapacityPlan::random(self)
    }

    fn fitness(&self, individual: &Self::Individual) -> f64 {
        let routing_and_capacity_plan = (individual as &dyn Any)
            .downcast_ref::<RoutingAndCapacityPlan>()
            .expect("Cannot downcast individual to sequence");
        -self.cost(routing_and_capacity_plan)
    }

    fn problem_settings(&self) -> ProblemSettings {
        self.problem_settings.clone()
    }
}

const DELAY_COST_COEFFICIENT: f64 = 100000000.0;

type Route = Vec<Id>;

#[derive(Debug, Clone)]
struct RoutingPlan {
    routes: Vec<Route>,
}

impl RoutingPlan {
    fn random_from_demands(network: &Network, demands: &Vec<BidirectionalDemand>) -> Self {
        let mut routes = Vec::new();
        for demand in demands {
            let (first, second) = demand.demand_nodes;
            let route = Self::any_route(network, first, second);
            routes.push(route);
        }
        assert!(
            routes.len() == demands.len(),
            "Number of routings and demands should be identical"
        );
        Self { routes }
    }

    /// Finds a route through a network.
    /// Nodes are not visited twice.
    /// **IMPORTANT:** No promise is made about the length of the route.
    /// # Arguments
    /// * `network` - A network to find a route in.
    /// * `from` - Start Id of the route to find.
    /// * `to` - End Id of the route to find.
    /// # Returns
    /// A `Route` from start to end, with all intermediate nodes.
    fn any_route(network: &Network, from: Id, to: Id) -> Route {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut path_sections = HashMap::new();
        queue.push_back(from);
        visited.insert(from);

        while let Some(current) = queue.pop_front() {
            if current == to {
                break;
            }
            let neighbors = network.neighbors(current);
            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    path_sections.insert(neighbor, current);
                    queue.push_back(neighbor);
                    visited.insert(neighbor);
                }
            }
        }

        let mut path = Vec::new();
        path.push(to);
        let mut current = to;
        while let Some(&previous) = path_sections.get(&current) {
            path.push(previous);
            current = previous;
        }
        path.reverse();
        assert_eq!(
            *path.first().expect("Path has no first element"),
            from,
            "First path element is wrong"
        );
        assert_eq!(
            *path.last().expect("Path has no last element"),
            to,
            "Last path element is wrong"
        );
        path
    }

    fn crossover(first: &Self, second: &Self) -> Self {
        let first_routes = &first.routes;
        let second_routes = &second.routes;
        assert!(
            first_routes.len() == second_routes.len(),
            "RoutingPlans must have the same number of routes"
        );
        let mut rng = rand::rng();
        let half_routing_length = first_routes.len() / 2;
        let first_crossover_index = rng.random_range(0..half_routing_length);
        let second_crossover_index = first_crossover_index + half_routing_length;
        let mut routes = Vec::new();
        for (i, (first_route, second_route)) in first_routes.iter().zip(second_routes).enumerate() {
            if i < first_crossover_index || i >= second_crossover_index {
                routes.push(first_route.clone());
            } else if i >= first_crossover_index && i < second_crossover_index {
                routes.push(second_route.clone());
            } else {
                panic!("Other indices should never be reached");
            }
        }
        Self { routes }
    }

    fn mutate(&mut self, problem: &Rofa) {
        let length = self.routes.len();
        let mut rng = rand::rng();
        let index_to_mutate = (0..length)
            .choose(&mut rng)
            .expect("Cannot choose index to mutate");
        let route_to_mutate = self
            .routes
            .get_mut(index_to_mutate)
            .expect("Cannot get route to mutate");
        let first = *route_to_mutate
            .first()
            .expect("Cannot get first in route to mutate");
        let second = *route_to_mutate
            .last()
            .expect("Cannot get last in route to mutate");
        let possible_intermediates: Vec<Id> = problem
            .network
            .nodes
            .iter()
            .map(|n| n.id)
            .filter(|&id| id != first && id != second)
            .collect();
        let intermediate = *possible_intermediates
            .choose(&mut rng)
            .expect("Cannot choose intermediate");
        let mut first_part = Self::any_route(&problem.network, first, intermediate);
        let second_part = Self::any_route(&problem.network, intermediate, second);
        first_part.pop();
        let mut mutated_route = Vec::new();
        mutated_route.extend(first_part);
        mutated_route.extend(second_part);
        *route_to_mutate = mutated_route;
    }
}

#[derive(Debug, Clone)]
struct CapacityPlan {
    planned_capacity_types: Vec<CapacityType>,
}

impl CapacityPlan {
    fn with_highest_capacities(capacity_types: &CapacityTypes, number_links: usize) -> Self {
        let planned_capacity_types = (0..number_links).map(|_| capacity_types.max()).collect();
        Self {
            planned_capacity_types,
        }
    }

    fn crossover(first: &Self, second: &Self) -> Self {
        assert!(
            first.planned_capacity_types.len() == second.planned_capacity_types.len(),
            "The length of capacity plans must match"
        );
        let length = first.planned_capacity_types.len();
        let length_other_parent = (length as f32 / 2.0).floor() as usize;
        let mut rng = rand::rng();
        let crossover_point_one = (0..length_other_parent)
            .choose(&mut rng)
            .expect("Could not choose first crossover point");
        let crossover_point_two = crossover_point_one + length_other_parent;
        let mut planned_capacity_types = Vec::new();
        for i in 0..length {
            if i <= crossover_point_one || i > crossover_point_two {
                planned_capacity_types.push(
                    first
                        .planned_capacity_types
                        .get(i)
                        .expect("Cannot access planned capacity types out of range")
                        .clone(),
                );
            } else if i > crossover_point_one && i <= crossover_point_two {
                planned_capacity_types.push(
                    second
                        .planned_capacity_types
                        .get(i)
                        .expect("Cannot access planned capacity types out of range")
                        .clone(),
                );
            } else {
                panic!("Logic error in two point crossover");
            }
        }
        Self {
            planned_capacity_types,
        }
    }

    fn mutate(&mut self, min_capacities: Vec<usize>, problem: &Rofa) {
        let length = self.planned_capacity_types.len();
        let mut rng = rand::rng();
        let index_to_mutate = (0..length)
            .choose(&mut rng)
            .expect("Cannot get index for mutation");
        let capacity_to_mutate = self
            .planned_capacity_types
            .get_mut(index_to_mutate)
            .expect("Cannot get capacity to mutate");
        let min_capacity = *min_capacities
            .get(index_to_mutate)
            .expect("Cannot get minimum required capacity");
        let mutated_capacity = problem
            .capacity_types
            .capacity_types
            .iter()
            .filter(|c| c.capacity > min_capacity)
            .choose(&mut rng)
            .expect("Cannot choose mutation capacity type")
            .clone();
        *capacity_to_mutate = mutated_capacity;
    }
}

#[derive(Debug, Clone)]
pub struct RoutingAndCapacityPlan {
    routing_plan: RoutingPlan,
    capacity_plan: CapacityPlan,
    links_demands: Vec<usize>,
    id: Id,
    parent_ids: Option<(Id, Id)>,
}

impl RoutingAndCapacityPlan {
    fn random(problem: &Rofa) -> Self {
        let routing_plan = RoutingPlan::random_from_demands(&problem.network, &problem.demands);
        // TODO: maybe store problem settings in rofa
        let capacity_plan = CapacityPlan::with_highest_capacities(
            &problem.capacity_types,
            problem.network.links.len(),
        );
        let links_demands = Self::calculate_links_demands(problem, &routing_plan);
        let id = get_id();
        let parent_ids = None;
        Self {
            routing_plan,
            capacity_plan,
            links_demands,
            id,
            parent_ids,
        }
    }

    pub fn get_links_demands(&self) -> Vec<usize> {
        self.links_demands.clone()
    }

    fn calculate_links_demands(problem: &Rofa, routing_plan: &RoutingPlan) -> Vec<usize> {
        let demands = &problem.demands;
        let network = &problem.network;
        let routes = &routing_plan.routes;

        let mut links_demands = vec![0; network.links.len()];
        for (demand, route) in demands.iter().zip(routes) {
            for i in 0..(route.len() - 1) {
                let link_ids = (
                    *route.get(i).expect("Cannot access route ids"),
                    *route.get(i + 1).expect("Cannot access route ids"),
                );
                let link_index = network.link_index_by_start_and_end_id(link_ids);
                let link_demand = links_demands
                    .get_mut(link_index)
                    .expect("Cannot get mutable link demand");
                *link_demand += demand.required_traffic;
            }
        }
        links_demands
    }
}

impl Individual for RoutingAndCapacityPlan {
    type Problem = Rofa;

    fn crossover(first: &Self, second: &Self, problem: &Self::Problem) -> Self {
        let routing_plan = RoutingPlan::crossover(&first.routing_plan, &second.routing_plan);
        let capacity_plan = CapacityPlan::crossover(&first.capacity_plan, &second.capacity_plan);
        let links_demands = RoutingAndCapacityPlan::calculate_links_demands(problem, &routing_plan);
        let id = get_id();
        let parent_ids = Some((first.id(), second.id()));
        Self {
            routing_plan,
            capacity_plan,
            links_demands,
            id,
            parent_ids,
        }
    }

    fn mutate(&mut self, problem: &Self::Problem) {
        self.routing_plan.mutate(problem);
        self.capacity_plan.mutate(self.get_links_demands(), problem);
    }

    fn id(&self) -> Id {
        self.id
    }

    fn parent_ids(&self) -> (Id, Id) {
        self.parent_ids.expect("Individuals has no parent ids")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coherent() {
        let nodes = (1..=5).map(Node::with_id).collect();
        let mut links = vec![
            BidirectionalLink::between((1, 2)),
            BidirectionalLink::between((1, 3)),
            BidirectionalLink::between((1, 4)),
            BidirectionalLink::between((2, 3)),
            BidirectionalLink::between((2, 4)),
        ];
        assert!(
            !Network::coherent(&nodes, &links),
            "Network should not be coherent"
        );
        links.push(BidirectionalLink::between((3, 5)));
        assert!(
            Network::coherent(&nodes, &links),
            "Network should be coherent"
        );
    }

    #[test]
    fn test_any_route() {
        let nodes = (1..=5).map(Node::with_id).collect();
        let links = vec![
            BidirectionalLink::between((1, 2)),
            BidirectionalLink::between((2, 3)),
            BidirectionalLink::between((3, 4)),
            BidirectionalLink::between((4, 5)),
        ];
        let network = Network::from(nodes, links);
        let path = RoutingPlan::any_route(&network, 1, 5);
        assert_eq!(
            path,
            vec![1, 2, 3, 4, 5],
            "Should find a path through the network"
        );
    }
}
