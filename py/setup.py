"""Setup script for MathCore Python package."""

from setuptools import setup, find_packages
from setuptools_rust import RustExtension

if __name__ == "__main__":
    setup(
        name="mathcore",
        version="0.6.0",
        packages=find_packages(where="py"),
        package_dir={"": "py"},
        rust_extensions=[
            RustExtension(
                "mathcore.mathcore_bridge",
                path="../crates/bridge/Cargo.toml",
                features=["pyo3"],
            ),
        ],
        zip_safe=False,
    )
