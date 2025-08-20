window.BENCHMARK_DATA = {
  "lastUpdate": 1755723192221,
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
      }
    ]
  }
}