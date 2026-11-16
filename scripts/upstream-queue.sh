#!/usr/bin/env bash
# upstream-queue.sh prs|issues  -- dump upstream open items as TSV: number, size/lines, title, labels. Retry-safe (gh is flaky).
source "$(dirname "$0")/lib.sh"; kind="${1:-prs}"
for i in 1 2 3; do
  if [ "$kind" = prs ]; then
    gh pr list -R Pumpkin-MC/Pumpkin --state open --limit 1000 --json number,title,additions,deletions,changedFiles,isDraft,mergeable,labels \
      --jq '.[]|[.number,(.additions+.deletions),.changedFiles,.mergeable,.isDraft,.title,([.labels[].name]|join(","))]|@tsv' && exit 0
  else
    gh issue list -R Pumpkin-MC/Pumpkin --state open --limit 1000 --json number,title,labels \
      --jq '.[]|[.number,.title,([.labels[].name]|join(","))]|@tsv' && exit 0
  fi; sleep 3
done; exit 1
