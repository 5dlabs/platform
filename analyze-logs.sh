#!/bin/bash

LOG_FILE="orchestrator-84d984586f-fmt9f.log"

echo "=== ORCHESTRATOR LOG ANALYSIS ==="
echo "Analyzing: $LOG_FILE"
echo ""

echo "1. ERROR MESSAGES:"
echo "=================="
grep -i "error\|ERROR" "$LOG_FILE" | grep -v "DEBUG" | head -20
echo ""

echo "2. FAILED STATUS UPDATES:"
echo "========================="
grep -i "failed.*status\|status.*failed" "$LOG_FILE" | head -10
echo ""

echo "3. JOB MONITORING ISSUES:"
echo "========================"
grep -i "job.*not found\|failed.*job\|job.*error" "$LOG_FILE" | head -10
echo ""

echo "4. API CALL FAILURES:"
echo "===================="
grep -i "patch.*failed\|failed.*patch\|api.*error\|kube.*error" "$LOG_FILE" | head -10
echo ""

echo "5. STATUS UPDATE ATTEMPTS:"
echo "=========================="
grep -i "status.*update\|update.*status\|patch_status" "$LOG_FILE" | head -15
echo ""

echo "6. JOB COMPLETION DETECTION:"
echo "============================"
grep -i "succeeded\|completed\|job.*complete\|completion_time" "$LOG_FILE" | head -10
echo ""

echo "7. RECONCILIATION ERRORS:"
echo "=========================="
grep -i "reconcil.*error\|reconcil.*failed\|error.*reconcil" "$LOG_FILE" | head -10
echo ""

echo "8. RECENT DOCS TASK (docs-gen-1753823439) EVENTS:"
echo "=================================================="
grep "docs-gen-1753823439" "$LOG_FILE" | grep -i "error\|failed\|status\|succeeded\|complete" | head -15
echo ""

echo "9. JOB OWNER REFERENCE ISSUES:"
echo "=============================="
grep -i "owner.*reference\|owner.*error" "$LOG_FILE" | head -5
echo ""

echo "10. FINALIZER ISSUES:"
echo "===================="
grep -i "finalizer.*error\|finalizer.*failed" "$LOG_FILE" | head -5
echo ""

echo "=== SUMMARY ANALYSIS ==="
echo "Total ERROR lines: $(grep -i "error\|ERROR" "$LOG_FILE" | wc -l)"
echo "Total status update lines: $(grep -i "status.*update\|update.*status" "$LOG_FILE" | wc -l)"
echo "Total reconciliation events: $(grep -i "reconciling object" "$LOG_FILE" | wc -l)"
echo "Recent docs task events: $(grep "docs-gen-1753823439" "$LOG_FILE" | wc -l)"