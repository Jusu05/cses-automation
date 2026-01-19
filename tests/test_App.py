from main import App, Settings
import pytest

class TestSettingsArument:
    @pytest.fixture
    def empty_temp_file(self, tmp_path):
        empty_temp_file = tmp_path / "test.ini"
        empty_temp_file.write_text("")

        return empty_temp_file

    @pytest.fixture
    def filled_temp_file(self, tmp_path):
        ini_content = f"""
        [general]
        cses_url = www.cses.fi

        [user]
        username = user
        password = 70617373776f7264
        """
                
        filled_temp_file = tmp_path / "test.ini"
        filled_temp_file.write_text(ini_content.strip())
        return filled_temp_file

    def test_set_one_setting(self, empty_temp_file):
        a = App(empty_temp_file)
        a.handle_settings(["--url", "www.cses.fi"])
        s = Settings(empty_temp_file)
        url = s.get_cses_url()
        assert url == "www.cses.fi"

    def test_edit_one_settings(self, filled_temp_file):
        a = App(filled_temp_file)
        a.handle_settings(["--url", "www.cses.fi"])
        s = Settings(filled_temp_file)
        url = s.get_cses_url()
        assert url == "www.cses.fi"

    def test_set_multiple_setting(self, empty_temp_file):
        a = App(empty_temp_file)
        a.handle_settings(["--username", "user1", "--password", "password1", "--url", "www.cses.fi/test"])
        s = Settings(empty_temp_file)
        url = s.get_cses_url()
        user, password = s.get_username_and_password()
        assert url == "www.cses.fi/test"
        assert user == "user1"
        assert password == "password1"
        
    def test_edit_multiple_settings(self, filled_temp_file):
        a = App(filled_temp_file)
        a.handle_settings(["--username", "user1", "--password", "password1", "--url", "www.cses.fi/test"])
        s = Settings(filled_temp_file)
        url = s.get_cses_url()
        user, password = s.get_username_and_password()
        assert url == "www.cses.fi/test"
        assert user == "user1"
        assert password == "password1"