# Time-Travel Versioning

Możliwość przywrócenia projektu do dowolnego stanu z historii.

```
Timeline:
v1 ─→ v2 ─→ v3 ─→ v4 ─→ v5 (HEAD)
 ↑                        ↑
 │                        └─ Current state
 └─ Can restore here!
```

## Kluczowe cechy

1. **Non-destructive**: Historia nigdy nie jest usuwana (unless GC)
2. **Fast**: Restore do HEAD = instant (O(1))
3. **Complete**: Cały stan projektu, nie tylko pojedyncze pliki