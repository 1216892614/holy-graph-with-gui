// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rayon::prelude::*;
use std::{fmt::Display, time::Instant};

const PRIME_LUT: [i32; 27] = [
    3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101, 103, 107,
];

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn compute(input_d6: String, input_lv: String) -> String {
    let mut input_d6: Vec<_> = input_d6
        .clone()
        .split("|")
        .map(|s| s.parse())
        .filter_map(|n| n.ok())
        .collect();

    let input_lv: usize = input_lv.parse().unwrap();

    let this_prime = [
        PRIME_LUT[(input_lv - 1) * 3 + 0],
        PRIME_LUT[(input_lv - 1) * 3 + 1],
        PRIME_LUT[(input_lv - 1) * 3 + 2],
    ];

    let max_dice_res = input_d6
        .iter()
        .sum::<i32>()
        .max(input_d6.iter().fold(1, |acc, x| acc * x));

    if this_prime.iter().all(|n| max_dice_res < *n) {
        return "N/A".to_owned();
    }

    input_d6.sort();

    let now = Instant::now();
    println!("task generating...");

    let calculated = bfs(input_d6);

    let first_stage_duration = now.elapsed();
    println!(
        "{} tasks total, cost {}s",
        calculated.len(),
        first_stage_duration.as_secs()
    );

    println!("computing...");

    let ans = calculated
        .into_par_iter()
        .map(|(pre_search, rest)| dfs(pre_search, rest, this_prime))
        .find_first(|ans| ans.is_some());

    let second_stage_duration = now.elapsed() - first_stage_duration;

    if let Some(Some(ans)) = ans {
        println!("res: {} = {}", ans, ans.compute().floor() as i32);
        println!(
            "Second stage: {}s. Total {}s",
            second_stage_duration.as_secs(),
            now.elapsed().as_secs()
        );

        return format!("{} = {}", ans, ans.compute().floor() as i32);
    }

    return "N/A".to_owned();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![compute])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug, Clone)]
enum AstNode {
    Add(Box<AstNode>, Box<AstNode>),
    Mul(Box<AstNode>, Box<AstNode>),
    Sub(Box<AstNode>, Box<AstNode>),
    Div(Box<AstNode>, Box<AstNode>),
    Num(i32),
}

impl AstNode {
    fn compute(&self) -> f32 {
        match self {
            AstNode::Add(l, r) => l.compute() + r.compute(),
            AstNode::Mul(l, r) => l.compute() * r.compute(),
            AstNode::Sub(l, r) => l.compute() - r.compute(),
            AstNode::Div(l, r) => l.compute() / r.compute(),
            AstNode::Num(n) => *n as f32,
        }
    }

    fn simplify(&self) -> String {
        let display = format!("{}", self);

        if !display.contains('*') && !display.contains('/') {
            return display.replace("(", "").replace(")", "");
        }

        display
    }
}

impl Display for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstNode::Add(l, r) => {
                write!(f, "({}+{})", l.simplify(), r.simplify())
            }
            AstNode::Sub(l, r) => {
                write!(f, "({}-{})", l.simplify(), r.simplify())
            }
            AstNode::Mul(l, r) => write!(f, "{}*{}", l, r),
            AstNode::Div(l, r) => write!(f, "{}/{}", l, r),
            AstNode::Num(n) => write!(f, "{}", n),
        }
    }
}

pub(crate) fn bfs(input_d6: Vec<i32>) -> Vec<(AstNode, Vec<i32>)> {
    fn search(this: i32, rest: Vec<i32>, depth: usize) -> Vec<(AstNode, Vec<i32>)> {
        if rest.len() <= 0 {
            return vec![(AstNode::Num(this), Vec::new())];
        }

        if depth > 2 {
            return vec![(AstNode::Num(this), rest)];
        }

        (0..rest.len())
            .into_iter()
            .map(|i| {
                let next = rest.get(i).unwrap().to_owned();
                let mut rest = rest.clone();

                rest.remove(i);

                search(next, rest, depth + 1)
            })
            .flatten()
            .map(|(possible_node, rest)| {
                let add = AstNode::Add(
                    Box::new(AstNode::Num(this)),
                    Box::new(possible_node.clone()),
                );
                let mul = AstNode::Mul(
                    Box::new(AstNode::Num(this)),
                    Box::new(possible_node.clone()),
                );
                let sub = AstNode::Sub(
                    Box::new(AstNode::Num(this)),
                    Box::new(possible_node.clone()),
                );
                let sub_rev = AstNode::Sub(
                    Box::new(possible_node.clone()),
                    Box::new(AstNode::Num(this)),
                );
                let div = AstNode::Div(
                    Box::new(AstNode::Num(this)),
                    Box::new(possible_node.clone()),
                );
                let div_rev = AstNode::Div(Box::new(possible_node), Box::new(AstNode::Num(this)));

                vec![
                    (add, rest.clone()),
                    (mul, rest.clone()),
                    (sub, rest.clone()),
                    (sub_rev, rest.clone()),
                    (div, rest.clone()),
                    (div_rev, rest.clone()),
                ]
            })
            .flatten()
            .collect()
    }

    if let Some(&next) = input_d6.get(1) {
        let mut rest = input_d6.clone();

        rest.remove(1);

        return search(next, rest, 0);
    }

    Vec::new()
}

pub(crate) fn dfs(pre_search: AstNode, rest: Vec<i32>, prime: [i32; 3]) -> Option<AstNode> {
    fn search(before: AstNode, rest: Vec<i32>, prime: [i32; 3]) -> Option<AstNode> {
        if rest.len() <= 0 && prime.contains(&(before.compute().floor() as i32)) {
            return Some(before);
        }

        if rest.len() <= 0 {
            return None;
        }

        let all_kind_rest: Vec<_> = (0..rest.len())
            .into_iter()
            .map(|i| {
                let next = rest.get(i).unwrap().to_owned();
                let mut rest = rest.clone();

                rest.remove(i);

                (next, rest)
            })
            .collect();

        for (this, rest) in all_kind_rest {
            if let Some(ans) = dfs(
                AstNode::Add(Box::new(before.clone()), Box::new(AstNode::Num(this))),
                rest.clone(),
                prime,
            ) {
                return Some(ans);
            }

            if let Some(ans) = dfs(
                AstNode::Mul(Box::new(before.clone()), Box::new(AstNode::Num(this))),
                rest.clone(),
                prime,
            ) {
                return Some(ans);
            }

            if let Some(ans) = dfs(
                AstNode::Sub(Box::new(AstNode::Num(this)), Box::new(before.clone())),
                rest.clone(),
                prime,
            ) {
                return Some(ans);
            }

            if let Some(ans) = dfs(
                AstNode::Sub(Box::new(before.clone()), Box::new(AstNode::Num(this))),
                rest.clone(),
                prime,
            ) {
                return Some(ans);
            }

            if let Some(ans) = dfs(
                AstNode::Div(Box::new(before.clone()), Box::new(AstNode::Num(this))),
                rest.clone(),
                prime,
            ) {
                return Some(ans);
            }

            if let Some(ans) = dfs(
                AstNode::Div(Box::new(AstNode::Num(this)), Box::new(before.clone())),
                rest,
                prime,
            ) {
                return Some(ans);
            }
        }

        return None;
    }

    search(pre_search, rest, prime)
}
