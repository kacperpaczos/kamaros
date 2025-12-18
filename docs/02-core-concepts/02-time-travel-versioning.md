# Time-Travel Versioning

> **Koncepcja cofania się w czasie do dowolnej wersji projektu**

[← Back: JCF Format](01-jcf-format.md) | [Next: Reverse Delta Strategy →](03-reverse-delta-strategy.md)

---

## Concept

Time-Travel Versioning = możliwość przywrócenia projektu do dowolnego stanu z historii.

```
Timeline:
v1 ─→ v2 ─→ v3 ─→ v4 ─→ v5 (HEAD)
 ↑                        ↑
 │                        └─ Current state
 └─ Can restore here!
```

## Key Features

1. **Non-destructive**: Historia nigdy nie jest usuwana (unless GC)
2. **Fast**: Restore do HEAD = instant (O(1))
3. **Complete**: Cały stan projektu, nie tylko pojedyncze pliki

## Implementation

See [`05-algorithms/02-restore-version.md`](../05-algorithms/02-restore-version.md) for details.

---

[← Back: JCF Format](01-jcf-format.md) | [Next: Reverse Delta Strategy →](03-reverse-delta-strategy.md)

