pub struct Asset {
    pub path: &'static str,
    pub content: &'static str,
}

macro_rules! asset {
    ($path:literal) => {
        Asset {
            path: $path,
            content: include_str!(concat!("../templates/v0.1.0/universal-typescript/", $path)),
        }
    };
}

pub const TEMPLATE_ASSETS: &[Asset] = &[
    asset!(".agents/plugins/marketplace.json.tpl"),
    asset!(".agents/skills/develop-template/SKILL.md.tpl"),
    asset!(".agents/skills/develop-template/references/compatibility.md.tpl"),
    asset!(".agents/skills/develop-template/scripts/check-package.mjs.tpl"),
    asset!(".agents/skills/develop-template/scripts/sync-metadata.mjs.tpl"),
    asset!(".claude-plugin/marketplace.json.tpl"),
    asset!(".claude-plugin/plugin.json.tpl"),
    asset!(".codex-plugin/plugin.json.tpl"),
    asset!(".github/workflows/npm-publish.yml.tpl"),
    asset!(".gitignore.tpl"),
    asset!("AGENTS.md.tpl"),
    asset!("README.md.tpl"),
    asset!("alchemy.json.tpl"),
    asset!("cordis.patch.yml.tpl"),
    asset!("mcp.json.tpl"),
    asset!("package.json.tpl"),
    asset!("plugin.json.tpl"),
    asset!("skills/shared-skill/SKILL.md.tpl"),
    asset!("skills/shared-skill/references/tool-contract.md.tpl"),
    asset!("skills/shared-skill/scripts/main.mjs.tpl"),
    asset!("skills/shared-skill/scripts/main.py.tpl"),
    asset!("skills/shared-skill/skill-runtime.json.tpl"),
    asset!("src/deepseek.ts.tpl"),
    asset!("src/opencode.ts.tpl"),
    asset!("tests/runtimes.test.mjs.tpl"),
    asset!("tsconfig.json.tpl"),
];

pub const GENERATED_VALIDATOR: &str = include_str!("../templates/v0.1.0/generated/validate.mjs");
pub const APACHE_LICENSE: &str = include_str!("../templates/v0.1.0/licenses/Apache-2.0.txt");
