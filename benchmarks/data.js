window.BENCHMARK_DATA = {
  "lastUpdate": 1755733667315,
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
      }
    ]
  }
}