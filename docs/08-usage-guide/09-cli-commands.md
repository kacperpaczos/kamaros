# CLI Commands

## Przegląd

Interfejs wiersza poleceń (CLI) Kamaros JCF Manager zapewnia pełne wsparcie dla wszystkich operacji dostępnych przez API klasy `JCFManager`. CLI jest zaprojektowany jako fasada dla biblioteki, umożliwiając pracę z projektami JCF bezpośrednio z terminala.

## Składnia ogólna

```bash
jcf <command> [options] [arguments]
```

## Dostępne komendy

### Zarządzanie projektem

#### `jcf init [directory]`
Inicjalizuje nowy projekt JCF w wskazanym katalogu (domyślnie: katalog bieżący).

```bash
# Inicjalizacja w katalogu bieżącym
jcf init

# Inicjalizacja w konkretnym katalogu
jcf init ./my-project

# Inicjalizacja z konfiguracją
jcf init --author "Jan Kowalski" --email "jan@example.com"
```

**Opcje:**
- `--author <name>` - domyślny autor commitów
- `--email <email>` - email autora
- `--compression-level <0-9>` - poziom kompresji (domyślnie: 6)
- `--max-history <MB>` - maksymalny rozmiar historii

#### `jcf config [key] [value]`
Wyświetla lub zmienia konfigurację projektu.

```bash
# Wyświetlenie całej konfiguracji
jcf config

# Wyświetlenie konkretnego ustawienia
jcf config author

# Zmiana ustawienia
jcf config author "Jan Kowalski"
jcf config compressionLevel 9
```

#### `jcf status`
Wyświetla status bieżącego projektu.

```bash
jcf status
```

Pokazuje:
- Bieżącą wersję (HEAD)
- Liczbę plików
- Rozmiar projektu
- Status zmian

### Operacje na plikach

#### `jcf add <file>...`
Dodaje pliki do projektu.

```bash
# Dodanie pojedynczego pliku
jcf add package.json

# Dodanie wielu plików
jcf add src/index.js src/utils.js

# Dodanie wszystkich plików w katalogu
jcf add src/

# Dodanie z opcją verbose
jcf add --verbose *.md
```

**Opcje:**
- `--verbose, -v` - szczegółowe informacje o postępie

#### `jcf remove <file>...`
Usuwa pliki z projektu.

```bash
# Usunięcie pojedynczego pliku
jcf remove old-file.js

# Usunięcie wielu plików
jcf remove temp/* cache/*
```

#### `jcf list [directory]`
Wyświetla listę plików w projekcie.

```bash
# Lista wszystkich plików
jcf list

# Lista plików w katalogu
jcf list src/

# Szczegółowa lista z rozmiarami
jcf list --long

# Lista z filtrami
jcf list --type js    # tylko pliki .js
jcf list --type md    # tylko pliki .md
```

**Opcje:**
- `--long, -l` - szczegółowa lista z rozmiarami i datami
- `--type <ext>` - filtrowanie po rozszerzeniu
- `--recursive, -r` - rekursywna lista

#### `jcf cat <file>`
Wyświetla zawartość pliku.

```bash
# Wyświetlenie zawartości pliku tekstowego
jcf cat README.md

# Wyświetlenie z numerami linii
jcf cat --number src/index.js

# Wyświetlenie binarnego pliku jako hex
jcf cat --hex image.png
```

**Opcje:**
- `--number, -n` - numery linii
- `--hex` - wyświetlenie jako hex dump
- `--encoding <enc>` - kodowanie (utf8, binary)

### Wersjonowanie

#### `jcf commit -m "message"`
Tworzy nowy checkpoint z bieżącymi zmianami.

```bash
# Prosty commit
jcf commit -m "Add login feature"

# Commit z autorem
jcf commit -m "Fix bug" --author "Jan Kowalski"

# Commit z opisem wieloliniowym
jcf commit -m "Implement user authentication

- Add login form
- Add password validation
- Add session management"
```

**Opcje:**
- `-m, --message <msg>` - wiadomość commit (wymagana)
- `--author <name>` - autor commit
- `--amend` - poprawka ostatniego commit

#### `jcf checkout <version>`
Przywraca projekt do wskazanej wersji.

```bash
# Przejście do konkretnej wersji
jcf checkout v1.2.3

# Przejście do wersji po ID
jcf checkout abc123def456

# Przejście do HEAD
jcf checkout HEAD

# Przejście do poprzedniej wersji
jcf checkout HEAD~1

# Przejście bez aktualizacji plików (tylko HEAD)
jcf checkout --detach v1.0
```

**Opcje:**
- `--detach` - odłączenie HEAD bez zmiany plików

#### `jcf show <version>`
Wyświetla szczegółowe informacje o konkretnej wersji.

```bash
# Szczegóły wersji
jcf show v1.0

# Szczegóły HEAD
jcf show HEAD

# Szczegóły z listą zmienionych plików
jcf show --stat v1.2.3
```

**Opcje:**
- `--stat` - statystyki zmian w wersji

#### `jcf log [options]`
Wyświetla historię wersji.

```bash
# Historia wszystkich wersji
jcf log

# Szczegółowa historia
jcf log --oneline

# Historia z autorem i datą
jcf log --pretty=format:"%h %s %an %ad"

# Historia konkretnego pliku
jcf log --follow src/index.js

# Ostatnie 5 wersji
jcf log -5
```

**Opcje:**
- `--oneline` - jednoliniowy format
- `--pretty <format>` - niestandardowy format
- `--follow <file>` - historia pliku
- `-n <count>` - limit wyników
- `--author <name>` - filtrowanie po autorze
- `--since <date>` - wersje od daty
- `--until <date>` - wersje do daty

#### `jcf diff [version1] [version2]`
Porównuje wersje lub pokazuje niezapisane zmiany.

```bash
# Zmiany od ostatniego commit
jcf diff

# Porównanie dwóch wersji
jcf diff v1.0 v1.1

# Porównanie z HEAD
jcf diff HEAD~1

# Szczegółowe porównanie
jcf diff --stat v1.0 v2.0
```

**Opcje:**
- `--stat` - statystyki zmian
- `--name-only` - tylko nazwy zmienionych plików

### Narzędzia konserwacji

#### `jcf gc`
Uruchamia garbage collection dla optymalizacji przestrzeni.

```bash
# Standardowe GC
jcf gc

# Szczegółowe informacje
jcf gc --verbose

# Symulacja (bez faktycznego usuwania)
jcf gc --dry-run
```

**Opcje:**
- `--verbose, -v` - szczegółowe informacje
- `--dry-run` - symulacja bez zmian
- `--aggressive` - agresywniejsze czyszczenie

#### `jcf verify`
Sprawdza integralność danych projektu.

```bash
# Weryfikacja całego projektu
jcf verify

# Weryfikacja konkretnej wersji
jcf verify v1.0

# Szczegółowa weryfikacja
jcf verify --verbose
```

**Opcje:**
- `--verbose, -v` - szczegółowe informacje
- `--fix` - automatyczna naprawa błędów (jeśli możliwe)

#### `jcf clean`
Czyści tymczasowe pliki i cache.

```bash
# Czyszczenie bezpieczne (tylko pliki tymczasowe)
jcf clean

# Agresywne czyszczenie (w tym cache)
jcf clean --cache

# Symulacja
jcf clean --dry-run
```

**Opcje:**
- `--cache` - usunięcie również cache
- `--dry-run` - symulacja bez usuwania

#### `jcf stats`
Wyświetla statystyki projektu.

```bash
# Ogólne statystyki
jcf stats

# Szczegółowe statystyki
jcf stats --detailed

# Statystyki konkretnej wersji
jcf stats v1.0
```

### Import/Eksport

#### `jcf export <file>`
Eksportuje projekt do pliku JCF.

```bash
# Eksport do pliku
jcf export my-project.jcf

# Eksport z postępem
jcf export --progress my-project.jcf

# Eksport konkretnej wersji
jcf export --version v1.0 my-project.jcf
```

**Opcje:**
- `--progress` - wyświetlanie postępu
- `--version <id>` - eksport konkretnej wersji

#### `jcf import <file>`
Importuje projekt z pliku JCF.

```bash
# Import projektu
jcf import my-project.jcf

# Import z postępem
jcf import --progress my-project.jcf

# Import do konkretnego katalogu
jcf import --target ./imported my-project.jcf
```

**Opcje:**
- `--progress` - wyświetlanie postępu
- `--target <dir>` - katalog docelowy

### Informacje i pomoc

#### `jcf help [command]`
Wyświetla pomoc.

```bash
# Ogólna pomoc
jcf help

# Pomoc dla konkretnej komendy
jcf help commit
jcf help add
```

#### `jcf version`
Wyświetla wersję JCF Manager.

```bash
jcf version
```

## Przykłady użycia

### Tworzenie nowego projektu

```bash
# Inicjalizacja
jcf init my-app
cd my-app

# Dodanie plików
echo '{"name": "my-app"}' > package.json
jcf add package.json

echo 'console.log("Hello");' > index.js
jcf add index.js

# Pierwszy commit
jcf commit -m "Initial setup"

# Dodanie więcej plików
mkdir src
echo 'function hello() { return "world"; }' > src/utils.js
jcf add src/utils.js
jcf commit -m "Add utility functions"
```

### Praca z wersjami

```bash
# Wyświetlenie historii
jcf log

# Przejście do starszej wersji
jcf checkout HEAD~1

# Powrót do najnowszej
jcf checkout HEAD

# Porównanie wersji
jcf diff HEAD~1 HEAD
```

### Zarządzanie projektem

```bash
# Sprawdzenie statusu
jcf status

# Lista plików
jcf list
jcf list --long

# Wyświetlenie zawartości
jcf cat package.json

# Optymalizacja
jcf gc
jcf verify
```

### Współpraca

```bash
# Eksport projektu
jcf export my-project-v1.jcf

# Udostępnienie komuś
# (przesłanie pliku my-project-v1.jcf)

# Import cudzego projektu
jcf init imported-project
cd imported-project
jcf import ../my-project-v1.jcf
```

## Kody wyjścia

- `0` - sukces
- `1` - błąd ogólny
- `2` - nieprawidłowe argumenty
- `3` - plik nie istnieje
- `4` - wersja nie istnieje
- `5` - błąd integralności danych

## Konfiguracja

CLI używa tych samych ustawień co API JCFManager. Konfiguracja może być zapisana w:

- Pliku `.jcf/config.json` w katalogu projektu
- Zmiennych środowiskowych (prefiks `JCF_`)
- Opcjach komend (najwyższy priorytet)

Przykład pliku konfiguracyjnego:

```json
{
  "author": "Jan Kowalski",
  "email": "jan@example.com",
  "compressionLevel": 6,
  "autoGC": true,
  "maxHistorySize": 100
}
```

## Integracja z powłoką

CLI wspiera autouzupełnianie w popularnych powłokach:

```bash
# Bash
source <(jcf completion bash)

# Zsh
source <(jcf completion zsh)

# Fish
jcf completion fish > ~/.config/fish/completions/jcf.fish
```