from main import IniParser
from pathlib import Path
import pytest

def test_return_file_path_pased_value():
        i = IniParser("test.ini")
        j = IniParser(Path("test.ini"))
        assert i.file == Path("test.ini")
        assert j.file == Path("test.ini")

def test_change_file_path():
    i = IniParser("test1.ini")
    i.file = "test2.ini"
    assert i.file == Path("test2.ini")

def test_type_error_on_invalid_type():
    i = IniParser("test.ini")
    with pytest.raises(TypeError):
        i.file = 1

def test_value_error_raised_on_invalid_file_suffix():
    i = IniParser("test.ini")
    with pytest.raises(ValueError):
        i.file = "test.py"

class TestReadinWriting:
    @pytest.fixture
    def temp_file(self, tmp_path):
        # Create a temporary INI file
        ini_content = """
        [DEFAULT]
        key1 = value1
        key2 = value2

        [Settings]
        setting1 = value1
        setting2 = value2
        """
        temp_file = tmp_path / "test.ini"
        temp_file.write_text(ini_content.strip())
        return temp_file

    def test_reading(self, temp_file):
        i = IniParser(temp_file)
        assert i.read("DEFAULT", "key1") == "value1"
        assert i.read("DEFAULT", "key2") == "value2"
        assert i.read("Settings", "setting1") == "value1"
        assert i.read("Settings", "setting2") == "value2"

    def test_editing(self, temp_file):
        i = IniParser(temp_file)
        i.edit("Settings", "setting1", "moi")
        x = i.read("Settings", "setting1")
        assert x == "moi"
    
    def test_write_new_section(self, temp_file):
        i = IniParser(temp_file)
        i.edit("m", "key1", "4")
        assert i.read("m", "key1") == "4"

    def test_non_string_writing(self, temp_file):
        i = IniParser(temp_file)
        i.edit("m", "key1", 4)
        assert i.read("m", "key1") == "4"
