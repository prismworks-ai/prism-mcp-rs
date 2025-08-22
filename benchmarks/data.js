window.BENCHMARK_DATA = {
  "lastUpdate": 1755892551933,
  "repoUrl": "https://github.com/prismworks-ai/prism-mcp-rs",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "committer": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "distinct": true,
          "id": "11e2bb1a40526c22a7a2603a1a8f919e36e790d7",
          "message": "fix: add write permissions to benchmark workflow\n\nThe benchmark workflow was failing with 'Permission denied' when trying\nto push benchmark results to the gh-pages branch. Added 'contents: write'\npermission to the benchmark job to allow GitHub Actions to push to gh-pages.\n\nFixes: Performance Benchmarks job 403 error when storing results",
          "timestamp": "2025-08-20T11:33:39-04:00",
          "tree_id": "15c1ac2e851f2089f10d68874b8728ee2893e4e5",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/11e2bb1a40526c22a7a2603a1a8f919e36e790d7"
        },
        "date": 1755704163003,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dummy_benchmark",
            "value": 100,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "committer": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "distinct": true,
          "id": "de11427748c3c60a047b4a4e455a7f449c92301c",
          "message": "fix: remove github-pages environment to resolve tag deployment restrictions\n\n- Remove environment protection rules that prevent tag deployments\n- Allows documentation deployment from release tags (e.g., v0.1.0)\n- Resolves: Tag 'v0.1.0' is not allowed to deploy to github-pages due to environment protection rules\n- Maintains all necessary permissions for GitHub Pages deployment",
          "timestamp": "2025-08-20T16:50:36-04:00",
          "tree_id": "1ba2eadd9af0dbe5fcd352bc05c4e2a05ea9997c",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/de11427748c3c60a047b4a4e455a7f449c92301c"
        },
        "date": 1755723191581,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dummy_benchmark",
            "value": 100,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "committer": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "distinct": true,
          "id": "b40a7148140144130f63e06b22daf10e0d8b4c52",
          "message": "fix: make release creation idempotent to handle existing releases\n\n- Check if release already exists before attempting to create\n- Prevents workflow failure when release is manually created\n- Improves release workflow reliability and enables rerunning",
          "timestamp": "2025-08-20T17:14:34-04:00",
          "tree_id": "4601bec56ff1e777f3e561553390cd4f59d098ec",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/b40a7148140144130f63e06b22daf10e0d8b4c52"
        },
        "date": 1755724632602,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dummy_benchmark",
            "value": 100,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "committer": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "distinct": true,
          "id": "fbfd7c4aa272f3ad2c865a7efb2eef8b21437996",
          "message": "fix: shorten keywords for crates.io compatibility\n\n- Replace 'model-context-protocol' (22 chars) with 'protocol' (8 chars)\n- Resolves crates.io publish error: keywords must have less than 20 characters\n- Maintains SEO and discoverability with core keywords",
          "timestamp": "2025-08-20T18:32:44-04:00",
          "tree_id": "3e33e85ce9f6d95b8ad347858ae8307fe9b7e381",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/fbfd7c4aa272f3ad2c865a7efb2eef8b21437996"
        },
        "date": 1755729307699,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dummy_benchmark",
            "value": 100,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "committer": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "distinct": true,
          "id": "8ba7c3c402d132849b2909614fe49c33c5c6a1f7",
          "message": "📝 Fix broken documentation link in README\n\n- Change 'Security Guide' link from docs/guides/security.md to docs/guides/authentication.md\n- The security.md file doesn't exist; authentication.md contains the auth/security content\n- All documentation links now point to existing files\n- Resolves broken link at bottom of README",
          "timestamp": "2025-08-20T19:21:35-04:00",
          "tree_id": "cc177b59aa492a9dc4345a24269807f1690e1983",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/8ba7c3c402d132849b2909614fe49c33c5c6a1f7"
        },
        "date": 1755732246796,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dummy_benchmark",
            "value": 100,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "committer": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "distinct": true,
          "id": "cb558e200b9f0b8bb3ee93e90edb4eae1009e9f1",
          "message": "📚 Enhance README with comprehensive documentation structure\n\n**Added Additional Documentation Links**:\n✅ Quick Start Guide - Installation and setup instructions\n✅ Plugin Types Reference - Detailed component specifications\n✅ Error Handling Guide - Comprehensive error management patterns\n✅ Development Setup - Development environment and workflows\n✅ Migration Guide - Migrating from other MCP implementations\n✅ Changelog - Version history and breaking changes\n✅ Security Policy - Vulnerability reporting procedures\n✅ Contributors - Recognition for project contributors\n\n**Improved Organization**:\n- Categorized documentation into logical sections with emojis\n- Added descriptive subtitles for better navigation\n- Enhanced Contributing section with Contributors link\n- Professional structure matching enterprise documentation standards\n\n**Impact**:\n- From 5 documentation links → 12 comprehensive guides\n- Better developer onboarding experience\n- Complete coverage of development lifecycle\n- Enterprise-grade documentation presentation\n\nAll links verified to point to existing, well-maintained files.",
          "timestamp": "2025-08-20T19:24:58-04:00",
          "tree_id": "eead24017267f3f657745524a9875efa4157d63c",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/cb558e200b9f0b8bb3ee93e90edb4eae1009e9f1"
        },
        "date": 1755732446644,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dummy_benchmark",
            "value": 100,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "committer": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "distinct": true,
          "id": "d06262673d2f33333b77c1c068531000e6283d9b",
          "message": "🚀 Add automatic crates.io publication to release workflow\n\n**New Features Added**:\n✅ Automatic crates.io publication on successful releases\n✅ Version consistency verification between Cargo.toml and release tag\n✅ Duplicate publication prevention with version checking\n✅ Post-publication verification with retry logic\n✅ Comprehensive release summary with status for all components\n\n**Workflow Enhancements**:\n- Added 'publish' job that runs after validate and release jobs\n- Uses existing CRATES_IO_TOKEN secret for authentication\n- Includes robust error handling and status reporting\n- Added 'summary' job providing complete release overview\n\n**Benefits**:\n- No more manual crates.io publication required\n- Automatic verification ensures reliability\n- Clear status reporting for all release components\n- Idempotent: safe to re-run without duplicate publications\n\n**Next Release Flow**:\n1. Tag release → 2. Platform validation → 3. GitHub release → 4. Documentation → 5. Crates.io publication → 6. Summary report",
          "timestamp": "2025-08-20T19:37:35-04:00",
          "tree_id": "adf240155bcfaefbb25bdef004103c501162b790",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/d06262673d2f33333b77c1c068531000e6283d9b"
        },
        "date": 1755733666357,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "client_create_default",
            "value": 86.986,
            "unit": "ns"
          },
          {
            "name": "client_create_with_config",
            "value": 123.97,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 104.88,
            "unit": "ns"
          },
          {
            "name": "500_bytes",
            "value": 430.28,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 238.73,
            "unit": "ns"
          },
          {
            "name": "plugin_config_empty",
            "value": 34.102,
            "unit": "ns"
          },
          {
            "name": "register_single_tool",
            "value": 346.64,
            "unit": "ns"
          },
          {
            "name": "lookup_in_10",
            "value": 19.432,
            "unit": "ns"
          },
          {
            "name": "lookup_in_100",
            "value": 21.098,
            "unit": "ns"
          },
          {
            "name": "lookup_in_1000",
            "value": 21.86,
            "unit": "ns"
          },
          {
            "name": "lookup_missing",
            "value": 20.376,
            "unit": "ns"
          },
          {
            "name": "execute_simple",
            "value": 116.48,
            "unit": "ns"
          },
          {
            "name": "execute_complex",
            "value": 434.58,
            "unit": "ns"
          },
          {
            "name": "create_metadata",
            "value": 107.19,
            "unit": "ns"
          },
          {
            "name": "state_update",
            "value": 186.35,
            "unit": "ns"
          },
          {
            "name": "generate_result",
            "value": 191.82,
            "unit": "ns"
          },
          {
            "name": "server_config_default",
            "value": 147.2,
            "unit": "ns"
          },
          {
            "name": "server_config_with_capabilities",
            "value": 466.03,
            "unit": "ns"
          },
          {
            "name": "route_simple",
            "value": 1.3772,
            "unit": "ns"
          },
          {
            "name": "route_complex",
            "value": 22.574,
            "unit": "ns"
          },
          {
            "name": "generate_success",
            "value": 209.91,
            "unit": "ns"
          },
          {
            "name": "generate_error",
            "value": 215.39,
            "unit": "ns"
          },
          {
            "name": "chain_3_middlewares",
            "value": 623.84,
            "unit": "ns"
          },
          {
            "name": "chain_5_middlewares",
            "value": 912.9,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "committer": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "distinct": true,
          "id": "45b136c38465f0b1075ba3cd7b3c7f7d770abf62",
          "message": "docs: Add comprehensive AI tool integration documentation\n\n- Add AI_TOOL_INTEGRATION.md with complete setup guides for Claude Desktop, Cursor, VS Code, Windsurf, and Claude Code\n- Add DEPLOYMENT_GUIDE.md with production deployment strategies, cross-platform builds, and distribution methods\n- Add TROUBLESHOOTING.md with systematic issue resolution and platform-specific solutions\n- Enhance GETTING_STARTED.md with end-to-end workflow and 5-minute quick start\n- Add examples/ai-tool-configs/ with configuration templates for all AI tools\n- Update main and docs README.md with improved navigation\n- Integrate plugin documentation with AI tool deployment workflow\n- Fix cross-links and validate all documentation references\n\nCloses the critical gap between server development and AI tool integration.\nTotal: 51,839 bytes of production-ready documentation added.",
          "timestamp": "2025-08-20T22:44:45-04:00",
          "tree_id": "26ef9d994a0ef31f2c0b2324e4d6bb3450f6eb9b",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/45b136c38465f0b1075ba3cd7b3c7f7d770abf62"
        },
        "date": 1755745206243,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "client_create_default",
            "value": 88.155,
            "unit": "ns"
          },
          {
            "name": "client_create_with_config",
            "value": 124,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 101.15,
            "unit": "ns"
          },
          {
            "name": "500_bytes",
            "value": 427.84,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 243.85,
            "unit": "ns"
          },
          {
            "name": "plugin_config_empty",
            "value": 34.617,
            "unit": "ns"
          },
          {
            "name": "register_single_tool",
            "value": 389.37,
            "unit": "ns"
          },
          {
            "name": "lookup_in_10",
            "value": 20.148,
            "unit": "ns"
          },
          {
            "name": "lookup_in_100",
            "value": 22.014,
            "unit": "ns"
          },
          {
            "name": "lookup_in_1000",
            "value": 23.202,
            "unit": "ns"
          },
          {
            "name": "lookup_missing",
            "value": 20.496,
            "unit": "ns"
          },
          {
            "name": "execute_simple",
            "value": 117.7,
            "unit": "ns"
          },
          {
            "name": "execute_complex",
            "value": 437.23,
            "unit": "ns"
          },
          {
            "name": "create_metadata",
            "value": 115.12,
            "unit": "ns"
          },
          {
            "name": "state_update",
            "value": 190.37,
            "unit": "ns"
          },
          {
            "name": "generate_result",
            "value": 224.11,
            "unit": "ns"
          },
          {
            "name": "server_config_default",
            "value": 151.04,
            "unit": "ns"
          },
          {
            "name": "server_config_with_capabilities",
            "value": 468.59,
            "unit": "ns"
          },
          {
            "name": "route_simple",
            "value": 1.3791,
            "unit": "ns"
          },
          {
            "name": "route_complex",
            "value": 22.698,
            "unit": "ns"
          },
          {
            "name": "generate_success",
            "value": 218.18,
            "unit": "ns"
          },
          {
            "name": "generate_error",
            "value": 217.15,
            "unit": "ns"
          },
          {
            "name": "chain_3_middlewares",
            "value": 625.52,
            "unit": "ns"
          },
          {
            "name": "chain_5_middlewares",
            "value": 893.29,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "committer": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "distinct": true,
          "id": "60e9fb5072c77bf36d3e029bd2e7560c102ed52d",
          "message": "fix: Resolve license and cargo-vet configuration issues\n\n- Add MPL-2.0 to allowed licenses in deny.toml for option-ext dependency\n- Reinitialize cargo-vet supply chain with compatible version\n- Replace incompatible wildcard version exemptions with proper format\n- All security checks now passing:\n  - cargo deny check licenses: ✅ PASS\n  - cargo vet --locked: ✅ PASS (388 exempted)\n\nFixes CI pipeline security validation failures.",
          "timestamp": "2025-08-20T23:11:20-04:00",
          "tree_id": "c6775874161bcf610472f9e5a710397330c00037",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/60e9fb5072c77bf36d3e029bd2e7560c102ed52d"
        },
        "date": 1755746505056,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "client_create_default",
            "value": 88.419,
            "unit": "ns"
          },
          {
            "name": "client_create_with_config",
            "value": 123.82,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 102.37,
            "unit": "ns"
          },
          {
            "name": "500_bytes",
            "value": 435.04,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 254.2,
            "unit": "ns"
          },
          {
            "name": "plugin_config_empty",
            "value": 34.655,
            "unit": "ns"
          },
          {
            "name": "register_single_tool",
            "value": 344.24,
            "unit": "ns"
          },
          {
            "name": "lookup_in_10",
            "value": 19.377,
            "unit": "ns"
          },
          {
            "name": "lookup_in_100",
            "value": 21.218,
            "unit": "ns"
          },
          {
            "name": "lookup_in_1000",
            "value": 22.127,
            "unit": "ns"
          },
          {
            "name": "lookup_missing",
            "value": 20.548,
            "unit": "ns"
          },
          {
            "name": "execute_simple",
            "value": 119.49,
            "unit": "ns"
          },
          {
            "name": "execute_complex",
            "value": 442.14,
            "unit": "ns"
          },
          {
            "name": "create_metadata",
            "value": 107.02,
            "unit": "ns"
          },
          {
            "name": "state_update",
            "value": 189.6,
            "unit": "ns"
          },
          {
            "name": "generate_result",
            "value": 192.6,
            "unit": "ns"
          },
          {
            "name": "server_config_default",
            "value": 146.81,
            "unit": "ns"
          },
          {
            "name": "server_config_with_capabilities",
            "value": 456.4,
            "unit": "ns"
          },
          {
            "name": "route_simple",
            "value": 1.3786,
            "unit": "ns"
          },
          {
            "name": "route_complex",
            "value": 22.756,
            "unit": "ns"
          },
          {
            "name": "generate_success",
            "value": 225.51,
            "unit": "ns"
          },
          {
            "name": "generate_error",
            "value": 215.13,
            "unit": "ns"
          },
          {
            "name": "chain_3_middlewares",
            "value": 668.15,
            "unit": "ns"
          },
          {
            "name": "chain_5_middlewares",
            "value": 918.46,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "committer": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "distinct": true,
          "id": "a597e2989dd65cc2fdfaf222c55136ea6509a099",
          "message": "fix: Restructure GitHub Pages deployment architecture\n\n- Create dedicated pages.yml workflow for proper Pages deployment\n- Separate benchmark generation from Pages deployment\n- Benchmarks now commit results to docs/benchmarks folder\n- Pages workflow deploys from docs/ folder with benchmark integration\n- Add professional landing page with project overview\n- Fix permissions and environment configuration\n- Should resolve all Pages deployment failures\n\nComplete separation of concerns for reliable documentation hosting.",
          "timestamp": "2025-08-20T23:39:25-04:00",
          "tree_id": "190d3ac71a39f05608e36bc013588206c475fe6d",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/a597e2989dd65cc2fdfaf222c55136ea6509a099"
        },
        "date": 1755748185458,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "client_create_default",
            "value": 88.551,
            "unit": "ns"
          },
          {
            "name": "client_create_with_config",
            "value": 124.51,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 101.92,
            "unit": "ns"
          },
          {
            "name": "500_bytes",
            "value": 436.63,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 240.13,
            "unit": "ns"
          },
          {
            "name": "plugin_config_empty",
            "value": 35.545,
            "unit": "ns"
          },
          {
            "name": "register_single_tool",
            "value": 377.77,
            "unit": "ns"
          },
          {
            "name": "lookup_in_10",
            "value": 19.208,
            "unit": "ns"
          },
          {
            "name": "lookup_in_100",
            "value": 21.663,
            "unit": "ns"
          },
          {
            "name": "lookup_in_1000",
            "value": 22.231,
            "unit": "ns"
          },
          {
            "name": "lookup_missing",
            "value": 20.292,
            "unit": "ns"
          },
          {
            "name": "execute_simple",
            "value": 113.84,
            "unit": "ns"
          },
          {
            "name": "execute_complex",
            "value": 448.04,
            "unit": "ns"
          },
          {
            "name": "create_metadata",
            "value": 107.31,
            "unit": "ns"
          },
          {
            "name": "state_update",
            "value": 191.43,
            "unit": "ns"
          },
          {
            "name": "generate_result",
            "value": 193.84,
            "unit": "ns"
          },
          {
            "name": "server_config_default",
            "value": 144.77,
            "unit": "ns"
          },
          {
            "name": "server_config_with_capabilities",
            "value": 456.24,
            "unit": "ns"
          },
          {
            "name": "route_simple",
            "value": 1.5548,
            "unit": "ns"
          },
          {
            "name": "route_complex",
            "value": 22.09,
            "unit": "ns"
          },
          {
            "name": "generate_success",
            "value": 222.31,
            "unit": "ns"
          },
          {
            "name": "generate_error",
            "value": 212.93,
            "unit": "ns"
          },
          {
            "name": "chain_3_middlewares",
            "value": 629.56,
            "unit": "ns"
          },
          {
            "name": "chain_5_middlewares",
            "value": 907.08,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "committer": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "distinct": true,
          "id": "cac8e60e1dad0f925151fad12de14f9f5e7ecf02",
          "message": "feat: Optimize crates.io keywords and description for AI/LLM discoverability\n\n- Update keywords to: ai, llm, agents, assistant, protocol\n- Enhance description to mention AI agents, LLM integrations, assistant tools\n- Optimize categories for better crates.io categorization\n- Improves discoverability for developers searching for AI/agent frameworks\n\nAligns crates.io metadata with GitHub topics for consistent search optimization.",
          "timestamp": "2025-08-20T23:55:20-04:00",
          "tree_id": "d36551250da4fdd6365885a5be335f6455a9cce8",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/cac8e60e1dad0f925151fad12de14f9f5e7ecf02"
        },
        "date": 1755749191881,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "client_create_default",
            "value": 88.329,
            "unit": "ns"
          },
          {
            "name": "client_create_with_config",
            "value": 123.98,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 102.5,
            "unit": "ns"
          },
          {
            "name": "500_bytes",
            "value": 430.08,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 240,
            "unit": "ns"
          },
          {
            "name": "plugin_config_empty",
            "value": 35.54,
            "unit": "ns"
          },
          {
            "name": "register_single_tool",
            "value": 351.68,
            "unit": "ns"
          },
          {
            "name": "lookup_in_10",
            "value": 19.787,
            "unit": "ns"
          },
          {
            "name": "lookup_in_100",
            "value": 21.293,
            "unit": "ns"
          },
          {
            "name": "lookup_in_1000",
            "value": 22.34,
            "unit": "ns"
          },
          {
            "name": "lookup_missing",
            "value": 20.436,
            "unit": "ns"
          },
          {
            "name": "execute_simple",
            "value": 113.38,
            "unit": "ns"
          },
          {
            "name": "execute_complex",
            "value": 441.06,
            "unit": "ns"
          },
          {
            "name": "create_metadata",
            "value": 107.09,
            "unit": "ns"
          },
          {
            "name": "state_update",
            "value": 191.34,
            "unit": "ns"
          },
          {
            "name": "generate_result",
            "value": 193.67,
            "unit": "ns"
          },
          {
            "name": "server_config_default",
            "value": 144.42,
            "unit": "ns"
          },
          {
            "name": "server_config_with_capabilities",
            "value": 448.79,
            "unit": "ns"
          },
          {
            "name": "route_simple",
            "value": 1.5538,
            "unit": "ns"
          },
          {
            "name": "route_complex",
            "value": 32.297,
            "unit": "ns"
          },
          {
            "name": "generate_success",
            "value": 208.57,
            "unit": "ns"
          },
          {
            "name": "generate_error",
            "value": 211.16,
            "unit": "ns"
          },
          {
            "name": "chain_3_middlewares",
            "value": 624.22,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "committer": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "distinct": true,
          "id": "8eaa0ed34b717c67987fc55bcedbd9a5fa99066c",
          "message": "fix: Improve release workflow reliability and timeouts\n\n🔧 Release workflow improvements:\n- Added 15-minute timeout to publish job to prevent indefinite hanging\n- Enhanced post-publication verification with better error handling\n- Improved cargo search pattern with timeout and regex matching\n- Added fallback verification via crates.io API\n- Reduced wait times (20s vs 30s) and max attempts (6 vs 10)\n- Better logging and user feedback during verification\n\n⚡ Performance enhancements:\n- Faster failure detection with timeouts\n- More reliable verification process\n- Graceful degradation when verification has issues\n\n🛡️ Reliability improvements:\n- Prevents workflows from hanging indefinitely\n- Better handling of crates.io propagation delays\n- Alternative verification methods for edge cases",
          "timestamp": "2025-08-22T11:10:46-04:00",
          "tree_id": "15762c047d84ed613ab947ad21514c60763bf018",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/8eaa0ed34b717c67987fc55bcedbd9a5fa99066c"
        },
        "date": 1755876041628,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "client_create_default",
            "value": 87.658,
            "unit": "ns"
          },
          {
            "name": "client_create_with_config",
            "value": 124.33,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 102.71,
            "unit": "ns"
          },
          {
            "name": "500_bytes",
            "value": 418.44,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 233.33,
            "unit": "ns"
          },
          {
            "name": "plugin_config_empty",
            "value": 32.127,
            "unit": "ns"
          },
          {
            "name": "register_single_tool",
            "value": 365.02,
            "unit": "ns"
          },
          {
            "name": "lookup_in_10",
            "value": 18.992,
            "unit": "ns"
          },
          {
            "name": "lookup_in_100",
            "value": 21.312,
            "unit": "ns"
          },
          {
            "name": "lookup_in_1000",
            "value": 22.458,
            "unit": "ns"
          },
          {
            "name": "lookup_missing",
            "value": 20.41,
            "unit": "ns"
          },
          {
            "name": "execute_simple",
            "value": 113.63,
            "unit": "ns"
          },
          {
            "name": "execute_complex",
            "value": 482.61,
            "unit": "ns"
          },
          {
            "name": "create_metadata",
            "value": 244.85,
            "unit": "ns"
          },
          {
            "name": "state_update",
            "value": 240.03,
            "unit": "ns"
          },
          {
            "name": "generate_result",
            "value": 275.38,
            "unit": "ns"
          },
          {
            "name": "server_config_default",
            "value": 147.45,
            "unit": "ns"
          },
          {
            "name": "server_config_with_capabilities",
            "value": 459.3,
            "unit": "ns"
          },
          {
            "name": "route_simple",
            "value": 1.5545,
            "unit": "ns"
          },
          {
            "name": "route_complex",
            "value": 22.394,
            "unit": "ns"
          },
          {
            "name": "generate_success",
            "value": 217.01,
            "unit": "ns"
          },
          {
            "name": "generate_error",
            "value": 214.78,
            "unit": "ns"
          },
          {
            "name": "chain_3_middlewares",
            "value": 622.95,
            "unit": "ns"
          },
          {
            "name": "chain_5_middlewares",
            "value": 882.8,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "committer": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "distinct": true,
          "id": "d5102061ae30027c1cfa2fd9aa3139da4706e636",
          "message": "fix: Resolve CI workflow test failures and compilation issues\n\n🧪 Test Fixes:\n- Fixed test_getting_started_examples.rs compilation errors\n- Corrected MCPServer -> McpServer naming convention\n- Fixed String parameter issues for McpServer::new()\n- Resolved McpError import and usage patterns\n- Fixed HashMap import conflicts\n- Proper error handling with McpError constructors\n\n📝 Code Quality:\n- Eliminated rustfmt formatting issues\n- Fixed all compilation errors in documentation examples\n- All 7 tests now pass successfully\n- Maintained backward compatibility with API examples\n\n⚡ Performance:\n- Tests run cleanly with only minor warnings\n- Documentation examples are now verified and functional\n- Improved reliability of CI test suite",
          "timestamp": "2025-08-22T11:30:52-04:00",
          "tree_id": "c609d780dab85351e22fcd393d0b8a32d380662e",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/d5102061ae30027c1cfa2fd9aa3139da4706e636"
        },
        "date": 1755877236254,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "client_create_default",
            "value": 87.563,
            "unit": "ns"
          },
          {
            "name": "client_create_with_config",
            "value": 124.14,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 102.48,
            "unit": "ns"
          },
          {
            "name": "500_bytes",
            "value": 429.91,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 235.42,
            "unit": "ns"
          },
          {
            "name": "plugin_config_empty",
            "value": 35.564,
            "unit": "ns"
          },
          {
            "name": "register_single_tool",
            "value": 342.28,
            "unit": "ns"
          },
          {
            "name": "lookup_in_10",
            "value": 19.162,
            "unit": "ns"
          },
          {
            "name": "lookup_in_100",
            "value": 21.122,
            "unit": "ns"
          },
          {
            "name": "lookup_in_1000",
            "value": 22.702,
            "unit": "ns"
          },
          {
            "name": "lookup_missing",
            "value": 20.357,
            "unit": "ns"
          },
          {
            "name": "execute_simple",
            "value": 112.63,
            "unit": "ns"
          },
          {
            "name": "execute_complex",
            "value": 454.87,
            "unit": "ns"
          },
          {
            "name": "create_metadata",
            "value": 108.54,
            "unit": "ns"
          },
          {
            "name": "state_update",
            "value": 189.66,
            "unit": "ns"
          },
          {
            "name": "generate_result",
            "value": 193.56,
            "unit": "ns"
          },
          {
            "name": "server_config_default",
            "value": 145.15,
            "unit": "ns"
          },
          {
            "name": "server_config_with_capabilities",
            "value": 456.8,
            "unit": "ns"
          },
          {
            "name": "route_simple",
            "value": 1.5535,
            "unit": "ns"
          },
          {
            "name": "route_complex",
            "value": 22.368,
            "unit": "ns"
          },
          {
            "name": "generate_success",
            "value": 210.38,
            "unit": "ns"
          },
          {
            "name": "generate_error",
            "value": 214.94,
            "unit": "ns"
          },
          {
            "name": "chain_3_middlewares",
            "value": 629.5,
            "unit": "ns"
          },
          {
            "name": "chain_5_middlewares",
            "value": 892.25,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "committer": {
            "email": "rishirandhawa@yahoo.com",
            "name": "Rishi",
            "username": "Rishirandhawa"
          },
          "distinct": true,
          "id": "7ae51bb2f8286b2a8d64736e52e019d9d406fd4c",
          "message": "🚀 FIX: Completely rebuild CI workflow to eliminate infrastructure failures\n\n✅ **MAJOR CI ROBUSTNESS IMPROVEMENTS**:\n\n## 🔧 **Cache/Archive Issues FIXED**:\n- Replace fragmented cache actions with unified Swatinem/rust-cache@v2\n- Implement cache corruption prevention with better keys\n- Add conditional cache saving to reduce conflicts\n- Include comprehensive cache directories for all Rust artifacts\n\n## 🎯 **Matrix Strategy FIXED**:\n- Add comprehensive job timeouts (45min main, step-specific)\n- Improve fail-fast strategy with smart exclusions\n- Remove problematic combinations (Windows beta/nightly, macOS beta)\n- Enhance job naming with OS/Rust version context\n\n## 🛠️ **Infrastructure Robustness ENHANCED**:\n- Add cargo retry configuration (3 attempts, 60s timeout)\n- Implement disk space cleanup for Ubuntu runners\n- Optimize checkout with minimal history (fetch-depth: 1)\n- Configure resource limits (2 parallel jobs) for stability\n\n## 🔍 **Error Handling IMPROVED**:\n- Add continue-on-error for non-critical steps\n- Implement artifact preservation with always() conditions\n- Create informative step names with emoji indicators\n- Add graceful degradation for missing directories\n\n## 📊 **Workflow Structure OPTIMIZED**:\n- **Required Jobs**: test, minimal, fmt, clippy, security, check\n- **Optional Jobs**: coverage, doc, examples, benchmarks\n- **CI Success Job**: Aggregate results with detailed status reporting\n- **Conditional Execution**: Benchmarks only on main branch\n\n## 🎉 **Expected Benefits**:\n- ✅ Eliminate \"Failed to delete archive\" errors\n- ⚡ Faster cache operations and reduced CI time\n- 🔍 Better error visibility and troubleshooting\n- 🚀 More reliable CI results for all contributors\n\n## 🎯 **Key Technical Improvements**:\n- Unified caching strategy prevents cache conflicts\n- Job-level and step-level timeouts prevent hanging\n- Resource management prevents disk space exhaustion\n- Retry mechanisms handle transient network issues\n- Clear separation of required vs optional validations\n\nThis addresses all infrastructure issues identified in the act simulation and\nprovides a production-ready, robust CI workflow that will work reliably on\nGitHub Actions without the cache/archive failures that caused matrix job issues.",
          "timestamp": "2025-08-22T15:45:55-04:00",
          "tree_id": "8890cfe17c9b942debada3cdaf89039ac7404517",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/7ae51bb2f8286b2a8d64736e52e019d9d406fd4c"
        },
        "date": 1755892551422,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "client_create_default",
            "value": 87.968,
            "unit": "ns"
          },
          {
            "name": "client_create_with_config",
            "value": 123.74,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 105.18,
            "unit": "ns"
          },
          {
            "name": "500_bytes",
            "value": 436.23,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 238.88,
            "unit": "ns"
          },
          {
            "name": "plugin_config_empty",
            "value": 35.09,
            "unit": "ns"
          },
          {
            "name": "register_single_tool",
            "value": 353.28,
            "unit": "ns"
          },
          {
            "name": "lookup_in_10",
            "value": 20.537,
            "unit": "ns"
          },
          {
            "name": "lookup_in_100",
            "value": 21.074,
            "unit": "ns"
          },
          {
            "name": "lookup_in_1000",
            "value": 22.098,
            "unit": "ns"
          },
          {
            "name": "lookup_missing",
            "value": 20.417,
            "unit": "ns"
          },
          {
            "name": "execute_simple",
            "value": 113.64,
            "unit": "ns"
          },
          {
            "name": "execute_complex",
            "value": 450.18,
            "unit": "ns"
          },
          {
            "name": "create_metadata",
            "value": 106.74,
            "unit": "ns"
          },
          {
            "name": "state_update",
            "value": 191.74,
            "unit": "ns"
          },
          {
            "name": "generate_result",
            "value": 201.32,
            "unit": "ns"
          },
          {
            "name": "server_config_default",
            "value": 145.69,
            "unit": "ns"
          },
          {
            "name": "server_config_with_capabilities",
            "value": 462.21,
            "unit": "ns"
          },
          {
            "name": "route_simple",
            "value": 1.5541,
            "unit": "ns"
          },
          {
            "name": "route_complex",
            "value": 22.075,
            "unit": "ns"
          },
          {
            "name": "generate_success",
            "value": 221.47,
            "unit": "ns"
          },
          {
            "name": "generate_error",
            "value": 216.09,
            "unit": "ns"
          },
          {
            "name": "chain_3_middlewares",
            "value": 640.78,
            "unit": "ns"
          },
          {
            "name": "chain_5_middlewares",
            "value": 909.94,
            "unit": "ns"
          }
        ]
      }
    ]
  }
}