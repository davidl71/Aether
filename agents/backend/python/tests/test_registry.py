from nautilus_backend.strategies.registry import default_registry


def test_default_registry_registers_example() -> None:
    registry = default_registry()
    factory = registry.get("example")
    strategy = factory()
    assert strategy.symbol == "ESZ4"
