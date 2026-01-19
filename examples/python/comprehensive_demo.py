#!/usr/bin/env python3
"""
Comprehensive Kamaros Demo - Full Workflow Example

This example demonstrates:
1. Creating a project with metadata
2. Adding text and binary files (images from internet)
3. Modifying files
4. Deleting files
5. Saving to .jcf archive
6. Loading in a new manager instance
7. Browsing version history
8. Rolling back to previous versions
"""

import os
import shutil
import urllib.request
from kamaros import JCFManager, FileAdapter

# === Configuration ===
PROJECT_STORE = "./demo-photo-album-store"
JCF_ARCHIVE = "./photo-album.jcf"


def download_image(url: str) -> bytes:
    """Download image from URL and return bytes."""
    print(f"    Downloading: {url}")
    with urllib.request.urlopen(url, timeout=10) as response:
        return response.read()


def cleanup():
    """Remove previous demo artifacts."""
    if os.path.exists(PROJECT_STORE):
        shutil.rmtree(PROJECT_STORE)
    if os.path.exists(JCF_ARCHIVE):
        os.remove(JCF_ARCHIVE)
    os.makedirs(PROJECT_STORE)


def print_separator(title: str):
    print(f"\n{'='*60}")
    print(f"  {title}")
    print(f"{'='*60}")


def print_history(manager: JCFManager):
    """Display version history."""
    manifest = manager.get_manifest()
    versions = manifest.get("versionHistory", [])
    print(f"\n  Historia wersji ({len(versions)} wersji):")
    for i, v in enumerate(versions):
        print(f"    [{i+1}] {v['id'][:8]}... | {v['message']} | {v['timestamp'][:19]}")


def main():
    cleanup()
    
    print_separator("FAZA 1: Tworzenie projektu")
    
    # Initialize manager with FileAdapter
    adapter = FileAdapter(PROJECT_STORE)
    manager = JCFManager(adapter)
    
    # Create project with full metadata
    manager.create_project(
        name="PhotoAlbum",
        description="Demonstracyjny album zdjęć z pełnym workflow",
        author="Kamaros Demo Script"
    )
    
    # Verify manifest has metadata
    info = manager.get_project_info()
    print(f"  Projekt: {info['name']}")
    print(f"  Plików: {info['file_count']}")
    
    # Add README
    readme_v1 = """# Photo Album

Album demonstracyjny dla biblioteki Kamaros.

## Zawartosc
- photo1.jpg - Losowe zdjecie 1
- photo2.jpg - Losowe zdjecie 2
""".encode("utf-8")
    manager.add_file("README.md", readme_v1)
    print("  Dodano: README.md")
    
    # Download and add images from picsum.photos (random images)
    print("  Pobieranie obrazów z internetu...")
    photo1 = download_image("https://picsum.photos/seed/kamaros1/400/300.jpg")
    photo2 = download_image("https://picsum.photos/seed/kamaros2/400/300.jpg")
    
    manager.add_file("images/photo1.jpg", photo1)
    manager.add_file("images/photo2.jpg", photo2)
    print(f"  Dodano: images/photo1.jpg ({len(photo1)} bytes)")
    print(f"  Dodano: images/photo2.jpg ({len(photo2)} bytes)")
    
    # Checkpoint v1
    v1_id = manager.save_checkpoint("Initial album with 2 photos")
    print(f"\n  ✓ Checkpoint v1: {v1_id[:8]}...")
    
    # =========================================================================
    print_separator("FAZA 2: Modyfikacje")
    
    # Update README
    readme_v2 = """# Photo Album

Album demonstracyjny dla biblioteki Kamaros.

## Zawartosc
- photo1.jpg - Losowe zdjecie 1 (zmodyfikowane)
- photo2.jpg - Losowe zdjecie 2
- photo3.jpg - NOWE! Trzecie zdjecie

## Zmiany
- Dodano photo3.jpg
- Zaktualizowano opisy
""".encode("utf-8")
    manager.add_file("README.md", readme_v2)
    print("  Zmodyfikowano: README.md")
    
    # Download different version of photo1 (simulate edit)
    photo1_modified = download_image("https://picsum.photos/seed/kamaros1mod/400/300.jpg")
    manager.add_file("images/photo1.jpg", photo1_modified)
    print(f"  Zmodyfikowano: images/photo1.jpg ({len(photo1_modified)} bytes)")
    
    # Add new photo
    photo3 = download_image("https://picsum.photos/seed/kamaros3/400/300.jpg")
    manager.add_file("images/photo3.jpg", photo3)
    print(f"  Dodano: images/photo3.jpg ({len(photo3)} bytes)")
    
    # Checkpoint v2
    v2_id = manager.save_checkpoint("Added photo3, updated descriptions")
    print(f"\n  ✓ Checkpoint v2: {v2_id[:8]}...")
    
    # =========================================================================
    print_separator("FAZA 3: Usuwanie pliku")
    
    manager.delete_file("images/photo2.jpg")
    print("  Usunięto: images/photo2.jpg")
    
    # Checkpoint v3
    v3_id = manager.save_checkpoint("Removed photo2")
    print(f"\n  ✓ Checkpoint v3: {v3_id[:8]}...")
    
    # Show current files
    files = manager.list_files()
    print(f"\n  Aktualne pliki: {files}")
    
    # =========================================================================
    print_separator("FAZA 4: Zapis do archiwum .jcf")
    
    archive_name = "photo-album.jcf"
    manager.save(archive_name)
    archive_path = os.path.join(PROJECT_STORE, archive_name)
    archive_size = os.path.getsize(archive_path)
    print(f"  Zapisano: {archive_path} ({archive_size:,} bytes)")
    
    # Clear reference
    del manager
    print("  Manager zamknięty.")
    
    # =========================================================================
    print_separator("FAZA 5: Wczytanie w nowym obiekcie")
    
    loaded_store = PROJECT_STORE + "-loaded"
    adapter2 = FileAdapter(loaded_store)
    os.makedirs(loaded_store, exist_ok=True)
    manager2 = JCFManager(adapter2)
    
    # Copy archive and blob storage to new adapter's location
    shutil.copy(archive_path, os.path.join(loaded_store, archive_name))
    store_src = os.path.join(PROJECT_STORE, ".store")
    store_dst = os.path.join(loaded_store, ".store")
    if os.path.exists(store_src):
        if os.path.exists(store_dst):
            shutil.rmtree(store_dst)
        shutil.copytree(store_src, store_dst)
    
    manager2.load(archive_name)
    print(f"  Wczytano: {archive_path}")
    
    info2 = manager2.get_project_info()
    print(f"  Projekt: {info2['name']}")
    print(f"  Wersji: {info2['version_count']}")
    print(f"  Plików: {info2['file_count']}")
    
    # =========================================================================
    print_separator("FAZA 6: Przeglądanie historii")
    
    print_history(manager2)
    
    # Show files in current state
    files = manager2.list_files()
    print(f"\n  Aktualne pliki: {files}")
    
    # =========================================================================
    print_separator("FAZA 7: Rollback - cofanie zmian")
    
    # Restore to v1 (should bring back photo2, remove photo3)
    print(f"\n  Przywracanie do v1 ({v1_id[:8]}...)...")
    manager2.restore_version(v1_id)
    
    files_v1 = manager2.list_files()
    print(f"  Pliki po przywróceniu v1: {files_v1}")
    
    # Verify photo2 is back
    photo2_restored = manager2.get_file("images/photo2.jpg")
    if photo2_restored:
        print(f"  ✓ photo2.jpg przywrócony ({len(photo2_restored)} bytes)")
    else:
        print("  ✗ BŁĄD: photo2.jpg nie został przywrócony!")
    
    # Verify photo3 is gone
    photo3_check = manager2.get_file("images/photo3.jpg")
    if photo3_check is None:
        print("  ✓ photo3.jpg usunięty (zgodnie z v1)")
    else:
        print("  ✗ BŁĄD: photo3.jpg nie powinien istnieć w v1!")
    
    # Now restore to v2 (photo3 should come back)
    print(f"\n  Przywracanie do v2 ({v2_id[:8]}...)...")
    manager2.restore_version(v2_id)
    
    files_v2 = manager2.list_files()
    print(f"  Pliki po przywróceniu v2: {files_v2}")
    
    photo3_restored = manager2.get_file("images/photo3.jpg")
    if photo3_restored:
        print(f"  ✓ photo3.jpg przywrócony ({len(photo3_restored)} bytes)")
    else:
        print("  ✗ BŁĄD: photo3.jpg nie został przywrócony!")
    
    # =========================================================================
    print_separator("PODSUMOWANIE")
    
    print("""
  Demonstracja zakończona pomyślnie!
  
  Przetestowano:
    ✓ Tworzenie projektu z metadanymi
    ✓ Dodawanie plików tekstowych i binarnych (obrazy z internetu)
    ✓ Modyfikacja istniejących plików
    ✓ Usuwanie plików
    ✓ Zapis do archiwum .jcf
    ✓ Wczytanie archiwum w nowej instancji
    ✓ Przeglądanie historii wersji
    ✓ Rollback do poprzednich wersji
    ✓ Weryfikacja poprawności przywracania
""")


if __name__ == "__main__":
    main()
