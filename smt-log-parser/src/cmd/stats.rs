use std::{collections::HashMap, path::PathBuf};

use petgraph::Direction;
use smt_log_parser::{
    analysis::{raw::IndexesInstGraph, InstGraph, RawNodeIndex},
    items::InstIdx,
    LogParser, Z3Parser,
};

pub fn run(logfile: PathBuf, top_k: Option<usize>) -> Result<(), String> {
    let path = std::path::Path::new(&logfile);
    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy())
        .unwrap_or_default();

    if !path.is_file() {
        return Err(format!("path {filename} did not point to a file"));
    }

    let (_metadata, parser) = Z3Parser::from_file(path).unwrap();

    let parser = parser.process_all().map_err(|e| e.to_string())?;
    let inst_graph = InstGraph::new(&parser).map_err(|e| format!("{e:?}"))?;

    let (no_enodes, no_geqs, no_treqs, no_insts) = {
        let mut no_enodes = 0;
        let mut no_given_equality = 0;
        let mut no_trans_equality = 0;
        let mut no_instantiations = 0;
        for ind in inst_graph.raw.node_indices() {
            match inst_graph.raw[ind].kind() {
                smt_log_parser::analysis::raw::NodeKind::ENode(_) => no_enodes += 1,
                smt_log_parser::analysis::raw::NodeKind::GivenEquality(_, _) => {
                    no_given_equality += 1;
                }
                smt_log_parser::analysis::raw::NodeKind::TransEquality(_) => no_trans_equality += 1,
                smt_log_parser::analysis::raw::NodeKind::Instantiation(_) => no_instantiations += 1,
            }
        }
        (
            no_enodes,
            no_given_equality,
            no_trans_equality,
            no_instantiations,
        )
    };

    let mut instantiations_occurrances: Vec<_> = {
        let mut count_mapping = HashMap::new();

        for (name, _) in parser
            .instantiations()
            .filter_map(|(idx, inst)| parser[inst.match_].kind.quant_idx().map(|v| (idx, v)))
            .filter_map(|(idx, quant_id)| {
                parser[quant_id].kind.user_name().map(|v| (&parser[v], idx))
            })
        {
            *count_mapping.entry(name).or_insert(0) += 1;
        }
        count_mapping.into_iter().map(|(k, v)| (v, k)).collect()
    };

    instantiations_occurrances.sort_by(|l: &(i32, &str), r: &(i32, &str)| Ord::cmp(&r.0, &l.0));

    println!("no-enodes: {}", no_enodes);
    println!("no-given-equalities: {}", no_geqs);
    println!("no-trans-equalities: {}", no_treqs);
    println!("no-instantiations: {}", no_insts);
    println!("nodes-count: {}", inst_graph.raw.graph.node_count());
    println!("top-instantiations=");
    let iter = instantiations_occurrances.iter();
    match top_k {
        None => {
            for (count, inst) in iter {
                println!("{} = {}", inst, count);
            }
        }
        Some(k) => {
            for (count, inst) in iter.take(k) {
                println!("{} = {}", inst, count);
            }
        }
    }

    Ok(())
}