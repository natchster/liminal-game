"""Dust-sheeted furniture: the chair (`sheeted.py`)."""
ORDER = 31


def build(kit):
    import sheeted
    return sheeted.build(kit, 'chair')
