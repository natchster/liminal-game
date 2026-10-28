"""Dust-sheeted furniture: the wingback (`sheeted.py`)."""
ORDER = 33


def build(kit):
    import sheeted
    return sheeted.build(kit, 'wingback')
