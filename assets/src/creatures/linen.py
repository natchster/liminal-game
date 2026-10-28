"""
The old linen of the dust sheets, the Watcher's and the furniture's alike
(at a glance one is the other): grey with handling, yellowed in blotches,
dirt rising from the hem, darker in the folds' hollows. Darker than it
looks: under the panels a white sheet blazes.
"""
import numpy as np


def colours(kit, o, uv, height):
    """per-vertex sRGB colours for sheet `o` (its vertices' places on the
    flat cloth `uv`, metres, and their heights above the floor)"""
    U = np.stack([uv[:, 0], np.zeros(len(uv)), uv[:, 1]], axis=1)
    linen = np.array(kit.srgb('#aaa394'))
    grey = np.array(kit.srgb('#867f72'))
    yellow = np.array(kit.srgb('#9e8f6c'))
    dirt = np.array(kit.srgb('#4f483f'))
    col = linen + (grey - linen) * np.clip(0.5 + kit.fbm(U, 1.6, 3, seed=71), 0, 1)[:, None] * 0.55
    blot = np.clip((kit.fbm(U, 2.3, 3, seed=72) - 0.18) * 4.0, 0, 1)
    col = col + (yellow - col) * blot[:, None] * 0.6
    hem = np.clip((0.45 + 0.12 * kit.fbm(U, 3.0, 2, seed=73) - height) / 0.45, 0, 1) ** 1.3
    col = col + (dirt - col) * hem[:, None] * 0.8
    cav = kit.cavity(o)
    return np.clip(col * (1 - 0.45 * cav)[:, None], 0.02, 0.95)
