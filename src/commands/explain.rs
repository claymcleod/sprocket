//! Implementation of the `explain` subcommand.

use std::collections::HashSet;

use anyhow::Ok;
use anyhow::anyhow;
use anyhow::bail;
use clap::Parser;
use colored::Colorize;
use wdl::analysis;
use wdl::lint;
use wdl::lint::Tag;

/// Usage string for the `explain` subcommand.
const USAGE: &str = "sprocket explain [RULE]
    sprocket explain --tag <TAG>
    sprocket explain --definitions";

/// Arguments for the `explain` subcommand.
#[derive(Parser, Debug)]
#[command(author, version, about, after_help = generate_after_help(), override_usage = USAGE)]
pub struct Args {
    /// The name of the rule to explain.
    #[arg(required_unless_present_any = [
        "tag",
        "definitions",
        "list_all_rules",
        "list_all_tags"
    ], value_name = "RULE")]
    pub rule_name: Option<String>,

    /// List all rules with the given tag.
    #[arg(short, long, value_name = "TAG", conflicts_with_all = ["rule_name", "definitions"])]
    pub tag: Option<String>,

    /// Display general WDL definitions.
    #[arg(long, conflicts_with_all = ["rule_name", "tag"])]
    pub definitions: bool,

    /// Lists all rules and exits.
    #[arg(long, conflicts_with_all = ["list_all_tags"])]
    pub list_all_rules: bool,

    /// Lists all tags and exits.
    #[arg(long, conflicts_with_all = ["list_all_rules"])]
    pub list_all_tags: bool,
}

/// Display all rules and tags.
fn generate_after_help() -> String {
    format!("{}\n\n{}", list_all_rules(), list_all_tags())
}

/// Lists all rules as a string for displaying.
pub fn list_all_rules() -> String {
    let mut result = String::from("Available rules:");
    let analysis_rules = analysis::rules();
    let lint_rules = lint::rules();

    let mut indexes = (0..(analysis_rules.len() + lint_rules.len())).collect::<Vec<_>>();

    let id = |index: usize| {
        if index >= analysis_rules.len() {
            lint_rules[index - analysis_rules.len()].id()
        } else {
            analysis_rules[index].id()
        }
    };

    indexes.sort_by(|a, b| id(*a).cmp(id(*b)));

    for index in indexes {
        result.push_str(&format!("\n  - {}", id(index)));
    }

    result
}

/// Lists all tags as a string for displaying.
pub fn list_all_tags() -> String {
    let mut result = String::from("Available tags:");
    let lint_rules = lint::rules();

    let mut tags: HashSet<Tag> = HashSet::new();
    for rule in lint_rules {
        for tag in rule.tags().iter() {
            tags.insert(tag);
        }
    }

    let mut tags: Vec<Tag> = tags.into_iter().collect();
    tags.sort_by_key(|tag| tag.to_string());

    for tag in tags {
        result.push_str(&format!("\n  - {}", tag));
    }

    result
}

/// Pretty prints a lint rule to a string.
pub fn pretty_print_lint_rule(rule: &dyn lint::Rule) {
    println!(
        "{id} {tags}",
        id = rule.id().bold().underline(),
        tags = format!("{}", rule.tags()).yellow()
    );
    println!("{desc}", desc = rule.description());
    println!("\n{explanation}", explanation = rule.explanation());

    if let Some(url) = rule.url() {
        println!("\n{url}", url = url.underline().blue());
    }

    let related = rule.related_rules();
    if !related.is_empty() {
        println!("\n{}", "Related Rules:".bold());
        let mut sorted_related = related.iter().collect::<Vec<_>>();
        sorted_related.sort();
        sorted_related.iter().for_each(|rule| {
            println!("  - {}", rule.cyan());
        });
    };
}

/// Pretty prints an analysis rule to a string.
pub fn pretty_print_analysis_rule(rule: &dyn analysis::Rule) {
    println!("{id}", id = rule.id().bold().underline());
    println!("{desc}", desc = rule.description());
    println!("\n{explanation}", explanation = rule.explanation());
}

/// Explains a lint rule.
pub fn explain(args: Args) -> anyhow::Result<()> {
    if args.list_all_rules {
        println!("{}", list_all_rules());
        return Ok(());
    }

    if args.list_all_tags {
        println!("{}", list_all_tags());
        return Ok(());
    }

    if args.definitions {
        println!("{}", lint::DEFINITIONS_TEXT);
        return Ok(());
    };

    if let Some(tag) = args.tag {
        let target = tag.parse::<Tag>().map_err(|_| {
            println!("{}\n", list_all_tags());
            anyhow!("invalid tag `{}`", tag)
        })?;

        let rules = lint::rules()
            .into_iter()
            .filter(|rule| rule.tags().contains(target))
            .collect::<Vec<_>>();

        if rules.is_empty() {
            println!("{}\n", list_all_tags());
            bail!("no rules found with the tag `{}`", tag);
        } else {
            println!("Rules with the tag `{}`:", tag);
            let mut rule_ids = rules.iter().map(|rule| rule.id()).collect::<Vec<_>>();
            rule_ids.sort();
            for id in rule_ids {
                println!("  - {}", id);
            }
        }
        return Ok(());
    }

    if let Some(rule_name) = args.rule_name {
        let lowercase_name = rule_name.to_lowercase();

        match analysis::rules()
            .into_iter()
            .find(|rule| rule.id().to_lowercase() == lowercase_name)
        {
            Some(rule) => {
                pretty_print_analysis_rule(rule.as_ref());
            }
            None => {
                match lint::rules()
                    .into_iter()
                    .find(|rule| rule.id().to_lowercase() == lowercase_name)
                {
                    Some(rule) => {
                        pretty_print_lint_rule(rule.as_ref());
                    }
                    None => {
                        println!("{rules}\n", rules = list_all_rules());
                        bail!("no rule found with the name `{rule_name}`");
                    }
                }
            }
        }

        return Ok(());
    }

    unreachable!();
}
