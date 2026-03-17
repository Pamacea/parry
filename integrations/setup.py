"""Parry Claude Code Hook - Python package"""

from setuptools import setup, find_packages
import os

# Read README for long description
def read_readme():
    readme_path = os.path.join(os.path.dirname(__file__), 'README.md')
    if os.path.exists(readme_path):
        with open(readme_path, 'r', encoding='utf-8') as f:
            return f.read()
    return ""

setup(
    name="parry-claude-hook",
    version="0.2.0",
    description="Claude Code integration hook for Parry linter",
    long_description=read_readme(),
    long_description_content_type="text/markdown",
    author="Parry Contributors",
    license="MIT",
    url="https://github.com/parry-org/parry",
    project_urls={
        "Bug Reports": "https://github.com/parry-org/parry/issues",
        "Source": "https://github.com/parry-org/parry",
    },
    packages=[],
    py_modules=["claude-hook"],
    entry_points={
        "console_scripts": [
            "parry-claude-hook=claude-hook:main",
        ],
    },
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
    ],
    python_requires=">=3.8",
    keywords="parry claude lint validation code-quality",
    install_requires=[
        # No external dependencies - uses only stdlib
    ],
    extras_require={
        "dev": [
            "pytest>=7.0",
            "pytest-asyncio>=0.21",
        ],
    },
)
