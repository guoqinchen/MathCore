"""Setup script for MathCore Python package."""

from setuptools import setup, find_packages

if __name__ == "__main__":
    setup(
        name="mathcore",
        version="0.6.0",
        packages=find_packages(where="."),
        package_dir={"": "py"},
    )
