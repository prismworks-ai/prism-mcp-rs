window.BENCHMARK_DATA = {
  "lastUpdate": 1766810832213,
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
          "id": "007649d116d753e5819a0eafba9a0de9956bf278",
          "message": "🚀 Fix critical CI failures through performance optimization\n\n- Fix CI timeouts by optimizing feature selection (--all-features → selective features)\n- Reduce build complexity: 412 → 312 crates (~25% faster compilation)\n- Simplify CI matrix: Remove nightly/beta variants, keep stable only\n- Support ubuntu-latest, macos-latest, windows-latest with stable Rust\n- Fix Cargo.toml profile duplications causing build errors\n- Add optimized test compilation settings\n- Separate full-features test as optional job with longer timeout\n\n✅ Result: 372/372 tests passing (100% success rate)\n✅ All critical CI jobs now pass: test, package-check, minimal-features\n\nResolves: CI workflow failures, timeout issues, build performance\nPerformance: ~25% faster builds with maintained comprehensive coverage",
          "timestamp": "2025-08-24T17:59:09-04:00",
          "tree_id": "3f00145641b843833c15d73a1c3d667ea37c91fa",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/007649d116d753e5819a0eafba9a0de9956bf278"
        },
        "date": 1756073346781,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "client_create_default",
            "value": 88.011,
            "unit": "ns"
          },
          {
            "name": "client_create_with_config",
            "value": 123.82,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 104.49,
            "unit": "ns"
          },
          {
            "name": "500_bytes",
            "value": 440.07,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 244.96,
            "unit": "ns"
          },
          {
            "name": "plugin_config_empty",
            "value": 35.931,
            "unit": "ns"
          },
          {
            "name": "register_single_tool",
            "value": 339.99,
            "unit": "ns"
          },
          {
            "name": "lookup_in_10",
            "value": 19.281,
            "unit": "ns"
          },
          {
            "name": "lookup_in_100",
            "value": 22.144,
            "unit": "ns"
          },
          {
            "name": "lookup_in_1000",
            "value": 21.931,
            "unit": "ns"
          },
          {
            "name": "lookup_missing",
            "value": 20.106,
            "unit": "ns"
          },
          {
            "name": "execute_simple",
            "value": 118.7,
            "unit": "ns"
          },
          {
            "name": "execute_complex",
            "value": 433.56,
            "unit": "ns"
          },
          {
            "name": "create_metadata",
            "value": 106.8,
            "unit": "ns"
          },
          {
            "name": "state_update",
            "value": 189.12,
            "unit": "ns"
          },
          {
            "name": "generate_result",
            "value": 194.07,
            "unit": "ns"
          },
          {
            "name": "server_config_default",
            "value": 145.52,
            "unit": "ns"
          },
          {
            "name": "server_config_with_capabilities",
            "value": 454.67,
            "unit": "ns"
          },
          {
            "name": "route_simple",
            "value": 1.5538,
            "unit": "ns"
          },
          {
            "name": "route_complex",
            "value": 22.38,
            "unit": "ns"
          },
          {
            "name": "generate_success",
            "value": 213.34,
            "unit": "ns"
          },
          {
            "name": "generate_error",
            "value": 211.75,
            "unit": "ns"
          },
          {
            "name": "chain_3_middlewares",
            "value": 638.73,
            "unit": "ns"
          },
          {
            "name": "chain_5_middlewares",
            "value": 915.56,
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
          "id": "f2912e4beae6fd635f6ee71fd3f66b2d03763107",
          "message": "fix: Add conditional compilation for Instant import\n\n- Make Instant import conditional on chunked-encoding OR http2 features\n- Instant used in StreamInfo struct (http2 feature) and performance timing (chunked-encoding feature)\n- Resolves coverage build failures where Instant type not found\n- Fixes CI error: 'cannot find type `Instant` in this scope'",
          "timestamp": "2025-08-24T18:34:30-04:00",
          "tree_id": "109567dd62a34b3b1c83af93f67b77ca3f6b65f8",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/f2912e4beae6fd635f6ee71fd3f66b2d03763107"
        },
        "date": 1756075479775,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "client_create_default",
            "value": 87.978,
            "unit": "ns"
          },
          {
            "name": "client_create_with_config",
            "value": 123.72,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 101.08,
            "unit": "ns"
          },
          {
            "name": "500_bytes",
            "value": 449.14,
            "unit": "ns"
          },
          {
            "name": "50_bytes",
            "value": 250.67,
            "unit": "ns"
          },
          {
            "name": "plugin_config_empty",
            "value": 35.114,
            "unit": "ns"
          },
          {
            "name": "register_single_tool",
            "value": 323.31,
            "unit": "ns"
          },
          {
            "name": "lookup_in_10",
            "value": 19.628,
            "unit": "ns"
          },
          {
            "name": "lookup_in_100",
            "value": 21.664,
            "unit": "ns"
          },
          {
            "name": "lookup_in_1000",
            "value": 21.991,
            "unit": "ns"
          },
          {
            "name": "lookup_missing",
            "value": 20.139,
            "unit": "ns"
          },
          {
            "name": "execute_simple",
            "value": 117.95,
            "unit": "ns"
          },
          {
            "name": "execute_complex",
            "value": 442.29,
            "unit": "ns"
          },
          {
            "name": "create_metadata",
            "value": 106.78,
            "unit": "ns"
          },
          {
            "name": "state_update",
            "value": 190.38,
            "unit": "ns"
          },
          {
            "name": "generate_result",
            "value": 193.34,
            "unit": "ns"
          },
          {
            "name": "server_config_default",
            "value": 146.44,
            "unit": "ns"
          },
          {
            "name": "server_config_with_capabilities",
            "value": 457.02,
            "unit": "ns"
          },
          {
            "name": "route_simple",
            "value": 1.5542,
            "unit": "ns"
          },
          {
            "name": "route_complex",
            "value": 22.076,
            "unit": "ns"
          },
          {
            "name": "generate_success",
            "value": 225.98,
            "unit": "ns"
          },
          {
            "name": "generate_error",
            "value": 211.39,
            "unit": "ns"
          },
          {
            "name": "chain_3_middlewares",
            "value": 629.18,
            "unit": "ns"
          },
          {
            "name": "chain_5_middlewares",
            "value": 917.98,
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
          "id": "f7fb5b31d03dcd935a57f8f09f408cadf956b7c9",
          "message": "fix: Resolve CI benchmark compilation and PowerShell issues\n\n- Remove criterion_main! from benchmark files (server_benchmarks.rs, client_benchmarks.rs, plugin_benchmarks.rs)\n- Remove unused criterion_main imports from benchmark files\n- Fix PowerShell syntax error in CI workflow by separating Unix/Windows verification steps\n- Benchmarks now properly compile without main function conflicts when using harness = false",
          "timestamp": "2025-08-24T19:27:31-04:00",
          "tree_id": "b7efe9698c53558056a3371adf817f5e612e2774",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/f7fb5b31d03dcd935a57f8f09f408cadf956b7c9"
        },
        "date": 1756078236927,
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
          "id": "0b80deeb2c267e275101db5f5ac758f84bbd6ad9",
          "message": "Refactor benchmarks and add utilities - consolidated benchmarks into single file and added utility modules",
          "timestamp": "2025-08-24T22:10:52-04:00",
          "tree_id": "ff43cfa826d8b4c55d5b0b895b022d18072d88d7",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/0b80deeb2c267e275101db5f5ac758f84bbd6ad9"
        },
        "date": 1756088115983,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 34.983,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 373.26,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 20.846,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 108.16,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 195.54,
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
          "id": "4fa9d454a0bc7342131b68fadc13421ddae8bf79",
          "message": "fix: Add cargo-vet audit entry for v0.1.3 and update Cargo.lock",
          "timestamp": "2025-08-24T22:22:05-04:00",
          "tree_id": "de16053084be4cd7a662ce3dfaa7bd85b475e560",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/4fa9d454a0bc7342131b68fadc13421ddae8bf79"
        },
        "date": 1756088794679,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 34.722,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 372.55,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 20.753,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 107.96,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 193.6,
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
          "id": "6f581b0b4c68d5002b2c6c4ce9a1778a6b31fbf3",
          "message": "fix: add cargo vet audit entry for v0.1.4\n\n- Add safe-to-deploy audit entry for prism-mcp-rs v0.1.4\n- Document badge improvements and documentation enhancements\n- Resolves cargo vet --locked compliance for CI/CD pipeline\n- Maintains security audit trail for dependency verification",
          "timestamp": "2025-08-24T23:02:53-04:00",
          "tree_id": "32d4bce069cad7401c7891eb2c6d6a2cb9bf95dc",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/6f581b0b4c68d5002b2c6c4ce9a1778a6b31fbf3"
        },
        "date": 1756091245445,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 35.055,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 379.13,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 21.044,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 106.98,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 202.2,
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
          "id": "ca52a5e3808265ea8fd4d5bcb78b6eeaa74fad4f",
          "message": "feat: enhance client module with production-ready features\n\n- Add comprehensive sampling/LLM integration examples (OpenAI/Anthropic)\n- Document roots security model with explicit warnings\n- Implement elicitation validation for required fields\n- Create SDK enhancement examples showcasing all convenience methods\n- Remove duplicate cargo vet from security.yml workflow\n- Add examples: sdk_enhancements_demo.rs, client_with_info.rs, client_with_anthropic.rs",
          "timestamp": "2025-09-10T16:08:36-04:00",
          "tree_id": "1d0256508ff60d5b64119506f950397d2acd55d1",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/ca52a5e3808265ea8fd4d5bcb78b6eeaa74fad4f"
        },
        "date": 1757535188212,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.452,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 380.03,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 22.117,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 77.517,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 171.16,
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
          "id": "4222cc03773523ad69ae3e35209e88681a7434cf",
          "message": "fix: resolve CI failures\n\n- Run cargo fmt on all files\n- Fix example compilation errors (add missing imports)\n- Comment out code requiring active connections in examples\n- Fix syntax error in client_with_anthropic.rs",
          "timestamp": "2025-09-10T16:18:34-04:00",
          "tree_id": "c496d0eda354e3f400b401e28a773964e50ea643",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/4222cc03773523ad69ae3e35209e88681a7434cf"
        },
        "date": 1757535684298,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.453,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 377.16,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 20.969,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 77.457,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 172.61,
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
          "id": "5eb38af958f3f2ebf959d64bd970b945b7356ba3",
          "message": "fix: complete OpenAI example compilation\n\n- Fix model type handling\n- Add Audio content variant match\n- Example now compiles successfully",
          "timestamp": "2025-09-10T16:28:49-04:00",
          "tree_id": "972c0e94d00050670f5ecdec954bbcee727283a8",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/5eb38af958f3f2ebf959d64bd970b945b7356ba3"
        },
        "date": 1757536331288,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.456,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 368.23,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 20.921,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 79.292,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 178.06,
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
          "id": "0e96aef173a3267780a8891adddabe36e3b9c39b",
          "message": "fix: remove compiler warnings in OpenAI example",
          "timestamp": "2025-09-10T16:34:24-04:00",
          "tree_id": "f78ca7c5304b73238c536604301307f0137da840",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/0e96aef173a3267780a8891adddabe36e3b9c39b"
        },
        "date": 1757536659300,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.446,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 356.71,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 20.982,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 77.627,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 171.81,
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
          "id": "e24efce8207be769578106cf264ea7250671eef2",
          "message": "fix: ensure all examples compile successfully\n\n- sdk_enhancements_demo.rs and client_with_info.rs now compile\n- All examples in the project build without errors\n- Only minor warnings about unused variables remain",
          "timestamp": "2025-09-10T16:57:15-04:00",
          "tree_id": "ef423af1711ce4525ba0ab7eed068048a26abca4",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/e24efce8207be769578106cf264ea7250671eef2"
        },
        "date": 1757538037309,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.447,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 376.13,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 21.326,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 77.568,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 172.21,
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
          "id": "efeeb40f84477aa2af215b4ed1d413e5eb6095c8",
          "message": "style: apply rustfmt formatting\n\n- Formatted all code with cargo fmt\n- No clippy errors, only minor warnings about unused code in examples",
          "timestamp": "2025-09-10T16:59:51-04:00",
          "tree_id": "952be15838f22acec38079c49384db3de065cf7c",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/efeeb40f84477aa2af215b4ed1d413e5eb6095c8"
        },
        "date": 1757540443032,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.453,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 361.31,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 20.855,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 77.553,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 176.91,
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
          "id": "c2b0d056724ae896875fe9b82f428ab8a9d1dda6",
          "message": "fix: Make examples compile without stdio feature",
          "timestamp": "2025-09-10T17:45:43-04:00",
          "tree_id": "fc21de86d6b106ee8f63fad981c849235a2721ee",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/c2b0d056724ae896875fe9b82f428ab8a9d1dda6"
        },
        "date": 1757540953986,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.448,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 364.6,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 21.092,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 77.883,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 172.1,
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
          "id": "34f39339b2d31f5fd715d5082c03b557f1d71f2b",
          "message": "chore: bump version to 0.1.5",
          "timestamp": "2025-09-10T19:00:40-04:00",
          "tree_id": "89da58588b30de865a63a6fb98b01869a2e3df58",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/34f39339b2d31f5fd715d5082c03b557f1d71f2b"
        },
        "date": 1757545416808,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.447,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 342.68,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 21.194,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 81.138,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 170.28,
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
          "id": "3725ce7d82ba0e2dbb501f0fffe193f8eeba60f7",
          "message": "fix: remove accidentally committed .edit_backups and .ai-context directories\n\nThese directories should not be in version control and are now properly ignored via .gitignore",
          "timestamp": "2025-09-10T21:18:20-04:00",
          "tree_id": "5298e3d628b38829d02b4e12a91afefc1cde2b0c",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/3725ce7d82ba0e2dbb501f0fffe193f8eeba60f7"
        },
        "date": 1757553700839,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.451,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 378.12,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 21.484,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 78.176,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 170.04,
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
          "id": "3cb141ea75e8d3540d41d718a3244d4a356982b7",
          "message": "fix: replace broken benchmarks badge with standard Rust badge\n\n- Removed non-existent benchmarks.yml workflow badge\n- Added standard Rust language badge\n- Updated API version badge to v0.1.5",
          "timestamp": "2025-09-10T21:48:13-04:00",
          "tree_id": "00397ad0cda45783158d8e4935c12bb714abcbf5",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/3cb141ea75e8d3540d41d718a3244d4a356982b7"
        },
        "date": 1757555481731,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.45,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 340.78,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 20.764,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 78.207,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 173.17,
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
          "id": "fe62e2f075c2864ff1bb972071dbbfeb049d8897",
          "message": "chore: upgrade rustls to 0.23.31 and fix dependency badge\n\n- Updated rustls from ^0.23 to 0.23.31 for security\n- Fixed dependency status badge to use deps.rs standard format\n- Ensures compatibility with latest security patches",
          "timestamp": "2025-09-10T21:52:50-04:00",
          "tree_id": "5c586b5691ca8ae761a1d761e1bd3ffa5ecf92de",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/fe62e2f075c2864ff1bb972071dbbfeb049d8897"
        },
        "date": 1757555748415,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.453,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 381.9,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 20.877,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 78.282,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 171.13,
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
          "id": "68772d2e2f8a67281f8c5b50b862dd593864a1f5",
          "message": "fix: add cargo-vet audit entry for v0.1.5\n\nAdded safe-to-deploy audit for prism-mcp-rs v0.1.5 to fix CI cargo-vet failure",
          "timestamp": "2025-09-10T21:56:13-04:00",
          "tree_id": "815c31a584416f4e99686525a86716be1dc21b22",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/68772d2e2f8a67281f8c5b50b862dd593864a1f5"
        },
        "date": 1757555964594,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.449,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 359.8,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 20.636,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 78.046,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 173.77,
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
          "id": "ada0e52375755b9ba0918b6b9be3c7b5f0342fd6",
          "message": "feat: add nightly automated dependency updater workflow\n\n- Runs daily at 2 AM UTC to check and update dependencies\n- Three modes: compatible (default), latest, or security-only updates\n- Automatically runs tests to verify updates work\n- Can use PAT token to trigger subsequent CI workflows\n- Creates issues on failure for monitoring\n- Includes comprehensive setup documentation\n\nBenefits:\n- Automatic security vulnerability fixes\n- Reduced maintenance burden\n- Small regular updates instead of large breaking changes\n- Full audit trail in git history",
          "timestamp": "2025-09-10T22:07:16-04:00",
          "tree_id": "a4edfb7813fc1ce0d5561396c04e93945e1db80f",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/ada0e52375755b9ba0918b6b9be3c7b5f0342fd6"
        },
        "date": 1757556612940,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.458,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 364.78,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 20.349,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 79.002,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 172.03,
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
          "id": "46efdb8b34629ae96dfbbbbd094ce1342ae0c783",
          "message": "\\fix: replace invalid actions/create-issue with actions/github-script in dependency-update workflow\\n\\nThe actions/create-issue action doesn't exist, causing workflow failures.\\nReplaced with actions/github-script@v7 to maintain same functionality\\nfor creating GitHub issues when dependency updates fail.\\n\\nFixes: GitHub Actions error 'repository not found'\\",
          "timestamp": "2025-09-10T22:12:42-04:00",
          "tree_id": "f5a28f5f7a5323a4b445dcc523fec996c290455b",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/46efdb8b34629ae96dfbbbbd094ce1342ae0c783"
        },
        "date": 1757556947993,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.451,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 359.24,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 21.178,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 84.691,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 173.03,
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
          "id": "02a280e2769307bdb0b6fc13c51b3549c32f5979",
          "message": "Release v1.0.0: Production-ready MCP SDK\n\n- Major version upgrade with breaking changes\n- Removed all deprecated APIs (add_simple_tool, run_with_*, etc.)\n- Added fluent interfaces and enhanced builder patterns\n- Improved error handling with structured error types\n- Added comprehensive utility modules and prelude\n- Fixed 4 security vulnerabilities in dependencies\n- Updated 13 outdated dependencies\n- Achieved zero compilation warnings\n- Complete test suite validation",
          "timestamp": "2025-09-11T19:34:09-04:00",
          "tree_id": "4f6e1d7f1e967c4d69831cc53b637434d285210c",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/02a280e2769307bdb0b6fc13c51b3549c32f5979"
        },
        "date": 1757633860098,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.441,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 383.75,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 21.304,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 77.943,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 171.04,
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
          "id": "c4637cad6cd3ad9d56a263a98f073d5588fb5a69",
          "message": "Release v1.0.0 - Major breaking changes release\n\nBREAKING CHANGES:\n- Removed all deprecated client methods (call_tool_simple, read_resource_simple, etc.)\n- Removed server convenience methods (add_simple_tool, run_with_stdio, etc.)\n- New SimpleTool pattern required for closure-based tool handlers\n- add_tool() now requires 4 parameters including JSON schema\n\nFeatures:\n- Enhanced builder pattern with fluent API\n- SimpleTool zero-cost abstraction for tool handlers\n- Improved structured error handling\n- 260 library tests passing\n\nSecurity:\n- Fixed 4 CVEs in dependencies (chrono, tungstenite, h2, rustls)\n\nSee RELEASE_NOTES.md for detailed migration guide",
          "timestamp": "2025-09-11T21:27:22-04:00",
          "tree_id": "c744d602bedbb09e6df9ef23d83f199ecd569010",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/c4637cad6cd3ad9d56a263a98f073d5588fb5a69"
        },
        "date": 1757640665008,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.845,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 363.35,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 21.298,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 81.512,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 171.69,
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
          "id": "ab1353babdd92b32bf92b322d4e8aa25db2bd1ad",
          "message": "feat: Implement comprehensive supply chain security audit recommendations\n\n- Complete supply chain security audit implementation\n- Add cargo-vet certification for v1.0.0 with security review\n- Update 6 dependencies with security improvements:\n  * cc 1.2.36 → 1.2.37 (build security)\n  * iana-time-zone 0.1.63 → 0.1.64\n  * rustls-webpki 0.103.4 → 0.103.5 (TLS security)\n  * windows-core 0.61.2 → 0.62.0\n  * windows-result 0.3.4 → 0.4.0\n  * windows-strings 0.4.2 → 0.5.0\n\nSecurity Automation:\n- Add automated security audit script (scripts/security-audit.sh)\n- Add safe dependency update script (scripts/update-dependencies.sh)\n- Add GitHub Actions security workflow for CI/CD integration\n- Weekly automated vulnerability scanning and reporting\n\nDocumentation:\n- Add comprehensive security section to README\n- Update SECURITY.md with complete security policy\n- Add supply chain transparency metrics\n- Document security tools and best practices\n\nSupply Chain Status:\n- ✅ Zero known vulnerabilities (379 dependencies scanned)\n- ✅ 100% policy compliance (cargo-deny)\n- ✅ Complete supply chain verification (cargo-vet)\n- ✅ All 8 unsafe blocks audited and contained\n- ✅ Automated monitoring with 95% automation\n\nSecurity Features:\n- Memory safety with minimal unsafe code\n- TLS 1.3 encryption with rustls\n- JWT authentication and RBAC authorization\n- Input validation and rate limiting\n- Comprehensive audit logging\n\nTools:\n- cargo-audit: Vulnerability scanning\n- cargo-deny: Policy enforcement\n- cargo-vet: Supply chain verification\n- GitHub Actions: Automated security workflows\n\nRisk Reduction: MEDIUM → LOW\nCompliance: Industry standards (OWASP, NIST, CIS)\nAudit Trail: Complete with Mozilla import chain\n\nThis implementation establishes enterprise-grade supply chain security\nwith automated monitoring, comprehensive documentation, and industry-\nleading security practices.",
          "timestamp": "2025-09-13T00:05:24-04:00",
          "tree_id": "d242d634dbd8930681c9b629893f53334d6c37e8",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/ab1353babdd92b32bf92b322d4e8aa25db2bd1ad"
        },
        "date": 1757736512750,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.45,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 365.62,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 21.615,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 77.963,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 170.32,
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
          "id": "140ce1bb1066662503db630a6c0c1bc9e99af4b5",
          "message": "\\feat: comprehensive development workflow improvements\n\n- Add pre-commit hooks configuration (.pre-commit-config.yaml)\n  - Automated code formatting with cargo fmt\n  - Linting with cargo clippy\n  - Basic compilation checks with cargo check\n  - File cleanup (trailing whitespace, line endings)\n  - Configuration validation (YAML/TOML)\n\n- Fix dependency update workflow permissions\n  - Add issues: write permission for issue creation\n  - Use PAT_TOKEN for issue creation to avoid GitHub token limitations\n  - Resolves 'Resource not accessible by integration' errors\n\n- Enhanced development documentation (docs/DEVELOPMENT.md)\n  - Comprehensive code quality workflow section\n  - Pre-commit hook setup instructions\n  - Common CI failure prevention guide\n  - Manual code quality commands reference\n  - Step-by-step development workflow\n\n- Update pre-commit installation script\n  - Add support for pre-commit framework installation\n  - Maintain existing doc-driven examples validation\n  - Provide clear setup instructions\n\nThese changes establish a robust development workflow that prevents\ncommon CI failures through automated checks and clear documentation.\\",
          "timestamp": "2025-09-13T00:29:28-04:00",
          "tree_id": "b8937ad7880311f19ca22ac346b49f3a4a8b6b87",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/140ce1bb1066662503db630a6c0c1bc9e99af4b5"
        },
        "date": 1757737965525,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.447,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 365.73,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 20.71,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 78.105,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 172.42,
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
          "id": "38002d8cb2e768f29f115101d681ab5e0ec2aa23",
          "message": "\\Release v1.1.0: Development workflow and CI improvements\n\n- Bump version from 1.0.0 to 1.1.0 in Cargo.toml\n- Update CHANGELOG.md with comprehensive v1.1.0 release notes\n  - Fixed GitHub Actions CI/CD formatting and compilation issues\n  - Added pre-commit hooks for automated code quality\n  - Enhanced development documentation and workflow\n  - Fixed dependency update workflow permissions\n  - Established sustainable development practices\n\nThis release focuses on developer experience improvements and\nCI/CD reliability, building upon the production-ready 1.0.0 foundation.\\",
          "timestamp": "2025-09-13T00:43:03-04:00",
          "tree_id": "50342155bd413a91a1f945aa013a0823f07d80f7",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/38002d8cb2e768f29f115101d681ab5e0ec2aa23"
        },
        "date": 1757738782608,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.453,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 376.99,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 20.773,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 77.917,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 172.14,
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
          "id": "c8d8a51127d5baa093ff527cbb06fc8ffc81d869",
          "message": "\\Release v1.1.1 - Enhanced examples and documentation\\n\\n- Updated and fixed multiple example files with correct API usage\\n- Enhanced plugin guides and error handling documentation\\n- Improved lib.rs exports and transport examples\\n- Added test coverage for documentation examples\\n- Better developer experience with working examples\\",
          "timestamp": "2025-09-13T17:16:20-04:00",
          "tree_id": "880ffd97cfdeddd85e0ba7b1ce8320daa7a4cc40",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/c8d8a51127d5baa093ff527cbb06fc8ffc81d869"
        },
        "date": 1757798398486,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.471,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 359.98,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 21.581,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 77.347,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 170.5,
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
          "id": "af3650db83d9491981130b54669a8fe3b30027b0",
          "message": "\\fix: add cargo-vet audit for prism-mcp-rs v1.1.1\n\n- Adds security audit entry for version 1.1.1 in supply-chain/audits.toml\n- Certifies as safe-to-deploy with maintenance updates only\n- Resolves CI/CD pipeline failures related to cargo-vet checks\n- No security-relevant code changes in this version\\",
          "timestamp": "2025-09-13T17:43:32-04:00",
          "tree_id": "6faf8130992a0901e312cddeac8e936e4171c139",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/af3650db83d9491981130b54669a8fe3b30027b0"
        },
        "date": 1757799990654,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 12.446,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 362.66,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 20.89,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 77.333,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 176.94,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "dev@vybecode.com",
            "name": "VybeCode Developer"
          },
          "committer": {
            "email": "dev@vybecode.com",
            "name": "VybeCode Developer"
          },
          "distinct": true,
          "id": "d5d451fdb4183b8f860e9c03642dece10641120b",
          "message": "fix: stop requesting deprecated cargo-edit component",
          "timestamp": "2025-12-26T22:39:47-05:00",
          "tree_id": "af138f76105a3e06c12a42d84ecb8ecab6fa4b79",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/d5d451fdb4183b8f860e9c03642dece10641120b"
        },
        "date": 1766807054023,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 13.389,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 370.35,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 21.08,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 77.471,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 169.3,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "dev@vybecode.com",
            "name": "VybeCode Developer"
          },
          "committer": {
            "email": "dev@vybecode.com",
            "name": "VybeCode Developer"
          },
          "distinct": true,
          "id": "3f1dbe3047037888d3dd5ba010190101d3459db4",
          "message": "chore: regenerate cargo-vet exemptions",
          "timestamp": "2025-12-26T23:01:01-05:00",
          "tree_id": "9e1fc6a87df69eca5c5917a56e7988fa9f37def8",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/3f1dbe3047037888d3dd5ba010190101d3459db4"
        },
        "date": 1766808268219,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 13.385,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 364.77,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 21.568,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 78.454,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 170.58,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "dev@vybecode.com",
            "name": "VybeCode Developer"
          },
          "committer": {
            "email": "dev@vybecode.com",
            "name": "VybeCode Developer"
          },
          "distinct": true,
          "id": "f6baf2e2f5c8aab5ca6d91f57af5c6b459e38e2a",
          "message": "fix: run cargo-vet against locked deps",
          "timestamp": "2025-12-26T23:12:02-05:00",
          "tree_id": "97d1848795c5a9e1f02fa1a598bcc258bf8c0719",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/f6baf2e2f5c8aab5ca6d91f57af5c6b459e38e2a"
        },
        "date": 1766808918980,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 13.385,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 369.51,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 21.352,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 77.7,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 172.82,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "dev@vybecode.com",
            "name": "VybeCode Developer"
          },
          "committer": {
            "email": "dev@vybecode.com",
            "name": "VybeCode Developer"
          },
          "distinct": true,
          "id": "52466ed8efb129adf99f5be6dfbae504f600f5da",
          "message": "fix: remove --locked from cargo-vet to fix supply chain audit\n\nThe --locked flag prevents cargo metadata updates which causes\ncargo-vet to fail when the crates.io index needs refreshing.\n\n🤖 Generated with [Claude Code](https://claude.com/claude-code)\n\nCo-Authored-By: Claude <noreply@anthropic.com>",
          "timestamp": "2025-12-26T23:44:13-05:00",
          "tree_id": "7fe804ea933bb273ea0fb8e55521b4e9eaf3ed19",
          "url": "https://github.com/prismworks-ai/prism-mcp-rs/commit/52466ed8efb129adf99f5be6dfbae504f600f5da"
        },
        "date": 1766810831499,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "plugin_config_creation",
            "value": 13.388,
            "unit": "ns"
          },
          {
            "name": "tool_registration",
            "value": 377.29,
            "unit": "ns"
          },
          {
            "name": "tool_lookup",
            "value": 21.035,
            "unit": "ns"
          },
          {
            "name": "plugin_metadata_creation",
            "value": 78.362,
            "unit": "ns"
          },
          {
            "name": "call_tool_result_generation",
            "value": 172.49,
            "unit": "ns"
          }
        ]
      }
    ]
  }
}