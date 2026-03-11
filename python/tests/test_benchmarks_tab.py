from types import SimpleNamespace

from python.tui.components.benchmarks_tab import _get_benchmarks_base_url


def test_benchmarks_base_url_defaults_to_shared_rust_origin():
    assert _get_benchmarks_base_url(SimpleNamespace(config=None)) == "http://127.0.0.1:8080"


def test_benchmarks_base_url_prefers_api_base_url():
    app = SimpleNamespace(config=SimpleNamespace(api_base_url="http://shared:8080/", backend_ports={}))

    assert _get_benchmarks_base_url(app) == "http://shared:8080"


def test_benchmarks_base_url_falls_back_to_shared_rust_origin_when_no_api_base():
    app = SimpleNamespace(
        config=SimpleNamespace(api_base_url=None, backend_ports={"risk_free_rate": 9004})
    )

    assert _get_benchmarks_base_url(app) == "http://127.0.0.1:8080"
