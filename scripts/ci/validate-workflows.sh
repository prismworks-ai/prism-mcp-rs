#!/bin/bash

# GitHub Actions Workflow Validation Script
# Validates YAML syntax and checks for common issues

set -e

echo "Search: Validating GitHub Actions workflows..."
echo

WORKFLOW_DIR=".github/workflows"
ERRORS=0

# Check if workflow directory exists
if [ ! -d "$WORKFLOW_DIR" ]; then
    echo "[!] Workflow directory not found: $WORKFLOW_DIR"
    exit 1
fi

# Function to validate YAML syntax
validate_yaml() {
    local file="$1"
    echo "  Checking $file..."
    
    # Basic YAML syntax check using python
    if python3 -c "import yaml; yaml.safe_load(open('$file'))" 2>/dev/null; then
        echo "    [x] YAML syntax valid"
    else
        echo "    [!] YAML syntax error"
        ((ERRORS++))
    fi
    
    # Check for required fields
    if grep -q "^name:" "$file" && grep -q "^on:" "$file" && grep -q "^jobs:" "$file"; then
        echo "    [x] Required fields present"
    else
        echo "    [!] Missing required fields (name, on, jobs)"
        ((ERRORS++))
    fi
    
    # Check for modern action versions
    if grep -q "actions/checkout@v[1-3]" "$file"; then
        echo "    Warning:  Consider updating checkout action to v4"
    fi
    
    echo
}

# Validate each workflow file
for workflow in "$WORKFLOW_DIR"/*.yml "$WORKFLOW_DIR"/*.yaml; do
    if [ -f "$workflow" ]; then
        validate_yaml "$workflow"
    fi
done

# Summary
echo "📊 Validation Summary:"
if [ $ERRORS -eq 0 ]; then
    echo "[x] All workflows are valid!"
    echo "# Ready to commit and test"
else
    echo "[!] Found $ERRORS error(s)"
    echo "- Please fix the issues before proceeding"
    exit 1
fi

echo
echo " Workflow inventory:"
ls -la "$WORKFLOW_DIR"/*.yml 2>/dev/null | awk '{print "  " $9 " (" $5 " bytes)"}'

echo
echo "## Next steps:"
echo "  1. Commit the changes"
echo "  2. Push to trigger workflow tests"
echo "  3. Monitor workflow executions"
echo "  4. Verify cache performance"
