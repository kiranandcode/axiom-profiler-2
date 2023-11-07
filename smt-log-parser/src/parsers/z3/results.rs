use std::collections::{BTreeMap, HashSet};

use petgraph::Graph;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use gloo_console::log;
use typed_index_collections::TiVec;

use crate::items::InstIdx;
use crate::items::Instantiation;

use super::z3parser::Z3Parser;

#[derive(Default)]
pub struct InstGraph {
    pub inst_graph: Graph::<usize, ()>, // weights are the line numbers and have type usize
    pub line_nr_of_node: BTreeMap<usize, usize>, // node-index => line number
    node_of_line_nr: BTreeMap<usize, NodeIndex>, // line number => node-index
}

impl InstGraph {
    fn fresh_line_nr(&self, line_nr: usize) -> bool {
        self.inst_graph.node_weights().all(|&line| line != line_nr)
    } 

    pub fn add_node(&mut self, line_nr: usize) {
        if self.fresh_line_nr(line_nr) {
            let node = self.inst_graph.add_node(line_nr);
            self.node_of_line_nr.insert(line_nr, node);
            self.line_nr_of_node.insert(node.index(), line_nr);
        }
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        if let (Some(&from_node_idx), Some(&to_node_idx)) = (self.node_of_line_nr.get(&from), self.node_of_line_nr.get(&to)) {
            self.inst_graph.add_edge(from_node_idx, to_node_idx, ());
        }
    }
}

impl Z3Parser {
    pub fn get_instantiation_graph(&self) -> InstGraph {
        // TODO: turn settings into an argument of this function
        let RenderSettings {max_line_nr, exclude_theory_solving_inst, max_instantiations} = RenderSettings::default();
        let mut graph = InstGraph::default(); 
        let mut insts: TiVec<InstIdx, Instantiation> = self.instantiations
            .iter()
            // only keep instantiations up to max_line_nr
            .filter(|inst| inst.line_no.is_some())
            .filter(|inst| inst.line_no.unwrap() <= max_line_nr)
            // if exlude_theory_solving_inst == true then only include inst if inst.quant_discovered = false 
            // since inst.quant_discovered == true iff inst is is a theory-solving inst (not due to mattern patch in e-graph) 
            .filter(|inst| !exclude_theory_solving_inst || !inst.quant_discovered)
            .cloned()
            .collect();

        // only keep the max_instantiations most expensive instantiations
        insts.sort_by(|inst1, inst2| inst2.cost.partial_cmp(&inst1.cost).unwrap());
        insts.truncate(max_instantiations);
        for inst in &insts {
            log!("Inst at line nr ", inst.line_no, " has cost ", inst.cost);
        } 
        let insts_lines: HashSet<usize> = insts.iter().filter_map(|inst| inst.line_no).collect();
        for dep in &self.dependencies {
            if dep.from > 0 {
                let from = dep.from;
                if let Some(to) = dep.to {
                    if insts_lines.contains(&from) && insts_lines.contains(&to) {
                        graph.add_node(from);
                        graph.add_node(to);
                        graph.add_edge(from, to);

                    }
                }
            }
        }
        graph
        
        // quant_discovered <=> instantiation not due to pattern-match in e-graph
        // for to_inst in insts.iter().filter(|inst| !inst.quant_discovered) {
        //     if let Some(to) = to_inst.line_no {
        //         graph.add_node(to);
        //         let from_iidxs = &to_inst.dep_instantiations;
        //         log!("The node at line nr ", to);
        //         for from_inst in from_iidxs.iter().filter_map(|&iidx| insts.get(iidx)) {
        //             if let Some(from) = from_inst.line_no {
        //                 if from > 0 {
        //                     log!("depends on the node at line nr ", from);
        //                     graph.add_node(from);
        //                     graph.add_edge(from, to);
        //                 }
        //             }
        //         }
        //     }
        // }
        // graph
    }
}

pub struct RenderSettings {
    max_line_nr: usize,
    exclude_theory_solving_inst: bool,
    max_instantiations: usize,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            max_line_nr: usize::MAX,
            exclude_theory_solving_inst: true,
            max_instantiations: 100,
        }
    }
}