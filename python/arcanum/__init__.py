# arcanum Python package
#
# Imports the compiled Rust extension (built with maturin) and re-exports
# public symbols into the arcanum namespace.
#
# Build the extension before importing:
#   maturin develop        (development build, editable install)
#   maturin build          (release wheel)

try:
    from .arcanum import *  # noqa: F401, F403
except ImportError as e:
    raise ImportError(
        "Arcanum extension module not found. "
        "Run 'maturin develop' from the repository root to build it."
    ) from e
