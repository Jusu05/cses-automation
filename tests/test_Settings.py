from main import Settings
import pytest

class TestGet:
    @pytest.fixture
    def empty_temp_file(self, tmp_path):
        empty_temp_file = tmp_path / "test.ini"
        empty_temp_file.write_text("")

        return empty_temp_file

    @pytest.fixture
    def filled_temp_file(self, tmp_path):
        ini_content = f"""
        [general]
        webdriver = {str(tmp_path / "geckodriver.exe")}
        cses_url = www.cses.fi

        [user]
        username = user
        password = 70617373776f7264

        [system]
        working_dir = {str(tmp_path)}
        """

        filled_temp_file = tmp_path / "test.ini"
        filled_temp_file.write_text(ini_content.strip())
        return filled_temp_file

    def test_get_webdriver_value_error_if_none(self, empty_temp_file):
        s = Settings(empty_temp_file)
        with pytest.raises(ValueError):
            s.get_webdriver_path()

    def test_get_cses_url_value_error_if_none(self, empty_temp_file):
        s = Settings(empty_temp_file)
        with pytest.raises(ValueError):
            s.get_cses_url()

    def test_get_username_and_password_value_error_if_none(self, empty_temp_file):
        s = Settings(empty_temp_file)
        with pytest.raises(ValueError):
            s.get_username_and_password()

    def test_get_working_dir_value_error_if_none(self, empty_temp_file):
        s = Settings(empty_temp_file)
        with pytest.raises(ValueError):
            s.get_working_dir()

    def test_get_webdriver(self, filled_temp_file, tmp_path):
        s = Settings(filled_temp_file)
        webdriver = s.get_webdriver_path()
        assert webdriver == str(tmp_path / "geckodriver.exe")

    def test_get_cses_url(self, filled_temp_file):
        s = Settings(filled_temp_file)
        url = s.get_cses_url()
        assert url == "www.cses.fi"

    def test_get_username_and_password(self, filled_temp_file):
        s = Settings(filled_temp_file)
        user, password = s.get_username_and_password()
        assert user == "user"
        assert password == "password"

    def test_get_working_dir(self, filled_temp_file, tmp_path):
        s = Settings(filled_temp_file)
        dir = s.get_working_dir()
        assert dir == str(tmp_path)


class TestSet:
    @pytest.fixture
    def empty_temp_file(self, tmp_path):
        empty_temp_file = tmp_path / "test.ini"
        empty_temp_file.write_text("")

        return empty_temp_file

    @pytest.fixture
    def temp_geckodriver1(self, tmp_path):
        temp_geckodriver = tmp_path / "geckodriver.exe"
        temp_geckodriver.write_text("")

        return temp_geckodriver
    
    @pytest.fixture
    def temp_geckodriver2(self, tmp_path):
        temp_geckodriver = tmp_path / "geckodriver"
        temp_geckodriver.write_text("")

        return temp_geckodriver

    @pytest.fixture
    def filled_temp_file(self, tmp_path, temp_geckodriver1):
        ini_content = f"""
        [general]
        webdriver = {temp_geckodriver1}
        cses_url = www.cses.fi

        [user]
        username = user
        password = 70617373776f7264

        [system]
        working_dir = {tmp_path}
        """
                
        filled_temp_file = tmp_path / "test.ini"
        filled_temp_file.write_text(ini_content.strip())
        return filled_temp_file

    def test_set_webdriver_path(self, empty_temp_file, temp_geckodriver1):
        s = Settings(empty_temp_file)
        driver = str(temp_geckodriver1)
        s.set_webdriver_path(driver)
        assert driver == s.get_webdriver_path() 

    def test_edit_webdriver_path(self, filled_temp_file, temp_geckodriver2):
        s = Settings(filled_temp_file)
        driver = str(temp_geckodriver2)
        s.set_webdriver_path(driver)
        assert driver == s.get_webdriver_path() 

    def test_set_webdriver_path_type_error_if_not_str(self, filled_temp_file):
        s = Settings(filled_temp_file)
        with pytest.raises(TypeError):
            s.set_webdriver_path(1)

    def test_set_webdriver_path_value_error_if_not_geckodirver(self, filled_temp_file):
        s = Settings(filled_temp_file)
        with pytest.raises(ValueError):
            s.set_webdriver_path("/home/test/ChromeDriver")

    def test_set_webdriver_path_file_not_found_if_not_exsis(self, filled_temp_file):
        s = Settings(filled_temp_file)
        with pytest.raises(FileNotFoundError):
            s.set_webdriver_path("/home/test/geckodriver")

    def test_set_cses_url(self, empty_temp_file):
        s = Settings(empty_temp_file)
        s.set_cses_url("cses.fi")
        assert "cses.fi" == s.get_cses_url() 

    def test_edit_cses_url(self, filled_temp_file):
        s = Settings(filled_temp_file)
        s.set_cses_url("cses.fi/test")
        assert "cses.fi/test" == s.get_cses_url() 
        
    def test_set_cses_url_type_eror_if_not_str(self, filled_temp_file):
        s = Settings(filled_temp_file)
        with pytest.raises(TypeError):
            s.set_cses_url(1)

    def test_set_cses_url_value_error_if_not_cses_url(self, filled_temp_file):
        s = Settings(filled_temp_file)
        with pytest.raises(ValueError):
            s.set_cses_url("google.com")

    def test_set_password_and_set_username(self, empty_temp_file):
        s = Settings(empty_temp_file)
        s.set_password("password")
        s.set_username("user")
        username, password = s.get_username_and_password()
        assert "password" == password 
        assert "user" == username 

    def test_edit_password_and_set_username(self, filled_temp_file):
        s = Settings(filled_temp_file)
        s.set_password("password1")
        s.set_username("user1")
        username, password = s.get_username_and_password()
        assert "password1" == password 
        assert "user1" == username 

    def test_set_pasword_type_error_if_not_str(self, filled_temp_file):
        s = Settings(filled_temp_file)
        with pytest.raises(TypeError):
            s.set_username(1)

    def test_set_pasword_type_error_if_not_str(self, filled_temp_file):
        s = Settings(filled_temp_file)
        with pytest.raises(TypeError):
            s.set_username(1)

    def test_set_username_type_error_if_not_str(self, filled_temp_file):
        s = Settings(filled_temp_file)
        with pytest.raises(TypeError):
            s.set_username(1)

    def test_set_workdir_type_path(self, empty_temp_file, tmp_path):
        s = Settings(empty_temp_file)
        s.set_working_dir(tmp_path)
        assert str(tmp_path) == s.get_working_dir()

    def test_set_workdir_type_str(self, empty_temp_file, tmp_path):
        s = Settings(empty_temp_file)
        s.set_working_dir(str(tmp_path))
        assert str(tmp_path) == s.get_working_dir()
    
    def test_edit_workdir_type_path(self, empty_temp_file, tmp_path):
        s = Settings(empty_temp_file)
        s.set_working_dir(tmp_path)
        assert str(tmp_path) == s.get_working_dir()

    def test_edit_workdir_type_str(self, empty_temp_file, tmp_path):
        s = Settings(empty_temp_file)
        s.set_working_dir(str(tmp_path))
        assert str(tmp_path) == s.get_working_dir()
    
    def test_set_workdir_type_error_if_not_str_or_path(self, filled_temp_file):
        s = Settings(filled_temp_file)
        with pytest.raises(TypeError):
            s.set_working_dir(1)
    
    def test_set_workdir_file_not_fuond_if_path_is_file(self, filled_temp_file):
        s = Settings(filled_temp_file)
        with pytest.raises(FileNotFoundError):
            s.set_working_dir(filled_temp_file)