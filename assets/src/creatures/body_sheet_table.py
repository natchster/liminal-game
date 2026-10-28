"""Dust-sheeted furniture: the table (`sheeted.py`)."""
ORDER = 32


def build(kit):
    import sheeted
    return sheeted.build(kit, 'table')
