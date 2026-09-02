from os.path import isdir, join
from platform import system
from sysconfig import get_config_var

from setuptools import Extension, find_packages, setup
from setuptools.command.build import build
from wheel.bdist_wheel import bdist_wheel


# True when the interpreter running setup.py is a free-threaded build. Such a
# build cannot use the stable ABI, because the module has to declare the
# Py_mod_gil slot and that slot is not part of the limited API.
FREE_THREADED = bool(get_config_var("Py_GIL_DISABLED"))

# CPython defines Py_GIL_DISABLED in pyconfig.h on POSIX free-threaded builds,
# but not on Windows, where it has to be passed to the compiler instead. Pass it
# unconditionally on free-threaded builds so binding.c sees the same macro
# everywhere; on POSIX this repeats the existing definition with the same value,
# which is a legal redefinition.
DEFINE_MACROS = [
    ("PY_SSIZE_T_CLEAN", None),
    ("TREE_SITTER_HIDE_SYMBOLS", None),
]
DEFINE_MACROS += (
    [("Py_GIL_DISABLED", "1")] if FREE_THREADED else [("Py_LIMITED_API", "0x03090000")]
)


class Build(build):
    def run(self):
        if isdir("queries"):
            dest = join(self.build_lib, "tree_sitter_ruby", "queries")
            self.copy_tree("queries", dest)
        super().run()


class BdistWheel(bdist_wheel):
    def get_tag(self):
        python, abi, platform = super().get_tag()
        if python.startswith("cp") and not FREE_THREADED:
            python, abi = "cp39", "abi3"
        return python, abi, platform


setup(
    packages=find_packages("bindings/python"),
    package_dir={"": "bindings/python"},
    package_data={
        "tree_sitter_ruby": ["*.pyi", "py.typed"],
        "tree_sitter_ruby.queries": ["*.scm"],
    },
    ext_package="tree_sitter_ruby",
    ext_modules=[
        Extension(
            name="_binding",
            sources=[
                "bindings/python/tree_sitter_ruby/binding.c",
                "src/parser.c",
                "src/scanner.c",
            ],
            extra_compile_args=[
                "-std=c11",
                "-fvisibility=hidden",
            ] if system() != "Windows" else [
                "/std:c11",
                "/utf-8",
            ],
            define_macros=DEFINE_MACROS,
            include_dirs=["src"],
            py_limited_api=not FREE_THREADED,
        )
    ],
    cmdclass={
        "build": Build,
        "bdist_wheel": BdistWheel
    },
    zip_safe=False
)
