#!/usr/bin/env python3
"""
Test Data Generators

Provides utilities for generating REAL test data (not mocks) for E2E tests.

Philosophy:
- Tests should generate their own data
- Data should be realistic and valid
- No pre-existing fixtures or hardcoded files
- Each test run creates fresh data

Usage:
    from orchestration.test_data.generators import AudioDataGenerator

    # In your E2E test:
    @pytest.fixture
    def test_audio_files():
        generator = AudioDataGenerator()
        files = generator.generate_wav_files(count=5, duration=1.0)
        yield files
        generator.cleanup()

    def test_cli_analyze(test_audio_files):
        # Now have 5 real audio files to test with
        result = subprocess.run([
            'python', '-m', 'cli', 'analyze', str(test_audio_files[0].parent)
        ], check=True)
"""

from pathlib import Path
from typing import List, Dict, Any, Callable
import tempfile
import json
import csv
import sqlite3


class TestDataGenerator:
    """Base class for test data generators."""

    def __init__(self, output_dir: Path = None):
        """
        Initialize generator.

        Args:
            output_dir: Directory to create files in.
                        If None, uses temporary directory.
        """
        if output_dir is None:
            self.output_dir = Path(tempfile.mkdtemp(prefix="test_data_"))
            self._cleanup_on_exit = True
        else:
            self.output_dir = Path(output_dir)
            self._cleanup_on_exit = False

        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.generated_files = []

    def cleanup(self):
        """Remove all generated files."""
        import shutil
        if self._cleanup_on_exit and self.output_dir.exists():
            shutil.rmtree(self.output_dir)


class AudioDataGenerator(TestDataGenerator):
    """Generate audio test files."""

    def generate_wav_files(
        self,
        count: int = 5,
        duration: float = 1.0,
        sample_rate: int = 44100
    ) -> List[Path]:
        """
        Generate WAV audio files.

        Args:
            count: Number of files to generate
            duration: Duration in seconds
            sample_rate: Sample rate in Hz

        Returns:
            List of generated file paths
        """
        import wave
        import struct
        import math

        files = []

        for i in range(count):
            filepath = self.output_dir / f"test_audio_{i}.wav"

            with wave.open(str(filepath), 'w') as wav:
                wav.setnchannels(1)  # Mono
                wav.setsampwidth(2)  # 16-bit
                wav.setframerate(sample_rate)

                # Generate sine wave
                frequency = 440.0 * (2 ** (i / 12.0))  # Different frequencies
                samples = int(sample_rate * duration)

                for j in range(samples):
                    value = int(32767 * math.sin(2 * math.pi * frequency * j / sample_rate))
                    wav.writeframes(struct.pack('<h', value))

            files.append(filepath)
            self.generated_files.append(filepath)

        return files

    def generate_mp3_files(
        self,
        count: int = 5,
        duration: float = 1.0
    ) -> List[Path]:
        """
        Generate MP3 audio files.

        Note: Requires pydub library. Falls back to WAV if unavailable.

        Args:
            count: Number of files to generate
            duration: Duration in seconds

        Returns:
            List of generated file paths
        """
        try:
            from pydub import AudioSegment
            from pydub.generators import Sine

            files = []

            for i in range(count):
                filepath = self.output_dir / f"test_audio_{i}.mp3"

                # Generate sine wave
                frequency = 440.0 * (2 ** (i / 12.0))
                sine_wave = Sine(frequency).to_audio_segment(duration=int(duration * 1000))

                # Export as MP3
                sine_wave.export(str(filepath), format="mp3")

                files.append(filepath)
                self.generated_files.append(filepath)

            return files

        except ImportError:
            print("Warning: pydub not available, falling back to WAV files")
            return self.generate_wav_files(count=count, duration=duration)


class DatabaseGenerator(TestDataGenerator):
    """Generate test databases."""

    def generate_sqlite_db(
        self,
        schema: Dict[str, List[Dict[str, Any]]],
        data: Dict[str, List[Dict[str, Any]]] = None,
        filename: str = "test_database.db"
    ) -> Path:
        """
        Generate SQLite database with schema and optional data.

        Args:
            schema: Dict of {table_name: [column_definitions]}
                    Example: {'users': [
                        {'name': 'id', 'type': 'INTEGER PRIMARY KEY'},
                        {'name': 'email', 'type': 'TEXT NOT NULL'}
                    ]}
            data: Dict of {table_name: [row_dicts]}
            filename: Database filename

        Returns:
            Path to generated database file
        """
        db_path = self.output_dir / filename
        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()

        # Create tables
        for table_name, columns in schema.items():
            col_defs = [f"{col['name']} {col['type']}" for col in columns]
            create_sql = f"CREATE TABLE {table_name} ({', '.join(col_defs)})"
            cursor.execute(create_sql)

        # Insert data if provided
        if data:
            for table_name, rows in data.items():
                if rows:
                    columns = list(rows[0].keys())
                    placeholders = ', '.join(['?' for _ in columns])
                    insert_sql = f"INSERT INTO {table_name} ({', '.join(columns)}) VALUES ({placeholders})"

                    for row in rows:
                        values = [row[col] for col in columns]
                        cursor.execute(insert_sql, values)

        conn.commit()
        conn.close()

        self.generated_files.append(db_path)
        return db_path


class CSVDataGenerator(TestDataGenerator):
    """Generate CSV test files."""

    def generate_csv(
        self,
        filename: str,
        headers: List[str],
        rows: int = 10,
        value_generators: Dict[str, Callable] = None
    ) -> Path:
        """
        Generate CSV file with synthetic data.

        Args:
            filename: Output filename
            headers: Column headers
            rows: Number of rows to generate
            value_generators: Dict of {column: generator_function}
                             Example: {'id': lambda i: i, 'name': lambda i: f'user_{i}'}

        Returns:
            Path to generated CSV file
        """
        filepath = self.output_dir / filename

        with open(filepath, 'w', newline='') as f:
            writer = csv.DictWriter(f, fieldnames=headers)
            writer.writeheader()

            for i in range(rows):
                row = {}
                for header in headers:
                    if value_generators and header in value_generators:
                        row[header] = value_generators[header](i)
                    else:
                        row[header] = f"{header}_{i}"

                writer.writerow(row)

        self.generated_files.append(filepath)
        return filepath


class JSONDataGenerator(TestDataGenerator):
    """Generate JSON test files."""

    def generate_json(
        self,
        filename: str,
        structure: Dict[str, Any]
    ) -> Path:
        """
        Generate JSON file.

        Args:
            filename: Output filename
            structure: Data structure to write

        Returns:
            Path to generated JSON file
        """
        filepath = self.output_dir / filename

        with open(filepath, 'w') as f:
            json.dump(structure, f, indent=2)

        self.generated_files.append(filepath)
        return filepath

    def generate_json_lines(
        self,
        filename: str,
        records: List[Dict[str, Any]]
    ) -> Path:
        """
        Generate JSON Lines (.jsonl) file.

        Args:
            filename: Output filename
            records: List of dictionaries (one per line)

        Returns:
            Path to generated JSONL file
        """
        filepath = self.output_dir / filename

        with open(filepath, 'w') as f:
            for record in records:
                f.write(json.dumps(record) + '\n')

        self.generated_files.append(filepath)
        return filepath


class TextDataGenerator(TestDataGenerator):
    """Generate text test files."""

    def generate_text_file(
        self,
        filename: str,
        lines: List[str] = None,
        line_count: int = 10,
        line_template: str = "Line {}"
    ) -> Path:
        """
        Generate text file.

        Args:
            filename: Output filename
            lines: Explicit lines to write (overrides line_count/template)
            line_count: Number of lines to generate
            line_template: Template for generated lines (uses .format(i))

        Returns:
            Path to generated text file
        """
        filepath = self.output_dir / filename

        with open(filepath, 'w') as f:
            if lines:
                f.write('\n'.join(lines) + '\n')
            else:
                for i in range(line_count):
                    f.write(line_template.format(i) + '\n')

        self.generated_files.append(filepath)
        return filepath


class ImageDataGenerator(TestDataGenerator):
    """Generate image test files."""

    def generate_png_images(
        self,
        count: int = 5,
        width: int = 100,
        height: int = 100,
        color: tuple = (255, 0, 0)
    ) -> List[Path]:
        """
        Generate PNG images.

        Note: Requires Pillow library.

        Args:
            count: Number of images to generate
            width: Image width in pixels
            height: Image height in pixels
            color: RGB color tuple (default: red)

        Returns:
            List of generated image paths
        """
        try:
            from PIL import Image

            files = []

            for i in range(count):
                filepath = self.output_dir / f"test_image_{i}.png"

                # Create solid color image
                img = Image.new('RGB', (width, height), color)
                img.save(filepath)

                files.append(filepath)
                self.generated_files.append(filepath)

            return files

        except ImportError:
            print("Warning: Pillow not available, cannot generate images")
            return []


# ============================================================================
# CONVENIENCE FUNCTIONS
# ============================================================================

def generate_test_audio_directory(count: int = 5) -> Path:
    """
    Quick function to generate directory with test audio files.

    Args:
        count: Number of audio files to generate

    Returns:
        Path to directory containing audio files
    """
    generator = AudioDataGenerator()
    generator.generate_wav_files(count=count)
    return generator.output_dir


def generate_test_csv_data(rows: int = 10, columns: List[str] = None) -> Path:
    """
    Quick function to generate test CSV file.

    Args:
        rows: Number of rows
        columns: Column names (default: ['id', 'name', 'value'])

    Returns:
        Path to generated CSV file
    """
    if columns is None:
        columns = ['id', 'name', 'value']

    generator = CSVDataGenerator()
    filepath = generator.generate_csv(
        'test_data.csv',
        headers=columns,
        rows=rows
    )
    return filepath


def generate_test_database(
    tables: Dict[str, List[str]] = None,
    rows_per_table: int = 10
) -> Path:
    """
    Quick function to generate test database.

    Args:
        tables: Dict of {table_name: [column_names]}
        rows_per_table: Number of rows to generate per table

    Returns:
        Path to generated database file
    """
    if tables is None:
        tables = {'test_table': ['id', 'name', 'value']}

    # Convert column names to schema
    schema = {}
    data = {}

    for table_name, columns in tables.items():
        schema[table_name] = [
            {'name': columns[0], 'type': 'INTEGER PRIMARY KEY'},
            *[{'name': col, 'type': 'TEXT'} for col in columns[1:]]
        ]

        data[table_name] = [
            {col: f'{col}_{i}' if i > 0 else i for i, col in enumerate(columns)}
            for i in range(rows_per_table)
        ]

    generator = DatabaseGenerator()
    db_path = generator.generate_sqlite_db(schema=schema, data=data)
    return db_path


# ============================================================================
# EXAMPLE USAGE
# ============================================================================

if __name__ == '__main__':
    """
    Example usage of test data generators.
    """
    print("Test Data Generators - Example Usage\n")

    # Audio files
    print("1. Generating audio files...")
    audio_gen = AudioDataGenerator()
    audio_files = audio_gen.generate_wav_files(count=3, duration=0.5)
    print(f"   Created {len(audio_files)} audio files in {audio_gen.output_dir}")

    # CSV file
    print("\n2. Generating CSV file...")
    csv_gen = CSVDataGenerator()
    csv_file = csv_gen.generate_csv(
        'sample.csv',
        headers=['id', 'name', 'score'],
        rows=5,
        value_generators={
            'id': lambda i: i,
            'name': lambda i: f'user_{i}',
            'score': lambda i: i * 10
        }
    )
    print(f"   Created CSV: {csv_file}")

    # Database
    print("\n3. Generating database...")
    db_gen = DatabaseGenerator()
    db_file = db_gen.generate_sqlite_db(
        schema={
            'users': [
                {'name': 'id', 'type': 'INTEGER PRIMARY KEY'},
                {'name': 'email', 'type': 'TEXT NOT NULL'},
                {'name': 'name', 'type': 'TEXT'}
            ]
        },
        data={
            'users': [
                {'id': 1, 'email': 'user1@example.com', 'name': 'User 1'},
                {'id': 2, 'email': 'user2@example.com', 'name': 'User 2'},
            ]
        }
    )
    print(f"   Created database: {db_file}")

    # JSON file
    print("\n4. Generating JSON file...")
    json_gen = JSONDataGenerator()
    json_file = json_gen.generate_json(
        'config.json',
        structure={
            'version': '1.0.0',
            'settings': {
                'debug': True,
                'max_retries': 3
            }
        }
    )
    print(f"   Created JSON: {json_file}")

    print("\n✅ All test data generated successfully")
    print("\nCleanup:")
    print(f"   Audio: {audio_gen.output_dir}")
    print(f"   CSV: {csv_gen.output_dir}")
    print(f"   Database: {db_gen.output_dir}")
    print(f"   JSON: {json_gen.output_dir}")
