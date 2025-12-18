# Garbage Collection

[← Back: Patch Application](04-patch-application.md) | [Next: File Rename Tracking →](06-file-rename-tracking.md)

## Algorithm: Mark & Sweep

1. **Mark**: Identify used blobs/deltas
2. **Sweep**: Find orphans
3. **Delete**: Remove unused
4. **Compact**: Repack ZIP

See IMPLEMENTATION_SPEC.md Section 5.3 for full algorithm.

[← Back: Patch Application](04-patch-application.md) | [Next: File Rename Tracking →](06-file-rename-tracking.md)
