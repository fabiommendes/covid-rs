try:
    from .epirust import * 
except ImportError:
    raise RuntimeError('compiled module not found')