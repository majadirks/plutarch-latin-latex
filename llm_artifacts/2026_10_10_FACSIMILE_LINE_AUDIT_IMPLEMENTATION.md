# Context

Implementation record for the source-line coverage and seam audit plan authored
2026-10-10. Production inputs and Latin remain unchanged. The preexisting
`BACKLOG.md` modification belongs to the user and is excluded from implementation
commits. Each completed required step receives its own commit and push.

# Progress

- 1.1: Extracted common infrastructure and a snapshot-independent occurrence
  mapper. Historical imports re-export existing names; historical mapping strips
  additional occurrence metadata. Output profiles protect input and archive paths;
  fixture PDF permission requires an exact root and filename allowlist. Imaging
  imports remain lazy. Compilation and current Tomus 1 mapping/path probes pass.
  Optional historical-map comparison requires an authenticated saved map.
- Phase 2: Deferred as explicitly permitted; cause diagnostics remain unknown.
