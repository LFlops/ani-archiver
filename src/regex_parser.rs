// src/regex_parser.rs

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::num::FpCategory::Infinite;

// 定义 .env 中规则的 JSON 结构
#[derive(Debug, Serialize, Deserialize)]
struct SubgroupRules {
    #[serde(flatten)]
    rules: HashMap<String, Vec<String>>,
}

// 全局静态缓存最终合并和编译后的正则表达式
// 优先加载 .env 中的规则，然后合并默认规则
lazy_static! {
    static ref COMPILED_RULES: HashMap<String, Vec<Regex>> = {
        let default_rules = {
            let mut map = HashMap::new();
            // 硬编码的默认规则
            map.insert("LoliSub".to_string(), vec![
                r"\[LoliSub\]\s*-\s*S(\d{2})E(\d{2})".to_string(),
                r"\[LoliSub\]\s*-\s*(\d{2})".to_string(),
            ]);
            map
        };

        // 尝试从 .env 中加载额外规则
        let external_rules_str = env::var("SUBGROUP_RULES").unwrap_or_else(|_| "{}".to_string());
        let external_rules: SubgroupRules = serde_json::from_str(&external_rules_str)
            .expect("Failed to parse SUBGROUP_RULES JSON from .env");

        // 合并规则：外部规则会覆盖同名的默认规则
        let mut final_rules = default_rules;
        for (name, rules) in external_rules.rules {
            final_rules.insert(name, rules);
        }

        // 编译所有规则
        let mut compiled_rules = HashMap::new();
        for (subgroup_name, regex_strs) in final_rules {
            let compiled_re = regex_strs.into_iter()
                .map(|s| Regex::new(&s).unwrap())
                .collect();
            compiled_rules.insert(subgroup_name, compiled_re);
        }
        compiled_rules
    };
}

/// 从文件名中提取季数和集数信息。
///
/// 该函数会按照以下顺序尝试匹配：
/// 1. 遍历所有已知的字幕组规则。
/// 2. 如果文件名包含某个字幕组的名称，就按顺序尝试该字幕组的所有规则。
///
/// # 参数
/// * `filename` - 视频文件的完整文件名。
///
/// # 返回
/// * `Some((season, episode))` - 如果匹配成功，返回季数和集数的元组。
/// * `None` - 如果没有任何规则匹配成功。
pub fn extract_episode_info(filename: &str) -> Option<(String, String)> {
    // 优先匹配包含字幕组名称的规则
    for (subgroup_name, rules) in COMPILED_RULES.iter() {
        // 如果字幕组不是 "Default" 且文件名包含该字幕组名称
        if !filename.contains(subgroup_name) {
            continue;
        }

        for re in rules {
            if let Some(caps) = re.captures(filename) {
                if caps.len() >= 3 {
                    let s_num = caps.get(1).unwrap().as_str();
                    let e_num = caps.get(2).unwrap().as_str();
                    return Some((s_num.to_string(), e_num.to_string()));
                } else if caps.len() == 2 {
                    let e_num = caps.get(1).unwrap().as_str();
                    return Some(("01".to_string(), e_num.to_string()));
                }
            }
        }
    }

    None
}
