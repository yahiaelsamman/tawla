"""Rule configuration for Mahbusa / Plakoto."""
from dataclasses import dataclass


@dataclass(frozen=True)
class Rules:
    """Rule switches. The defaults are Egyptian Mahbusa as confirmed for this project.

    mother_exception:
        False (Egyptian): pinning the opponent's last checker on their starting point
        ends the game at once.
        True (Greek Plakoto): the game continues if the pinner still has checkers on
        their own starting point; if both mothers end up pinned, the game is a draw.
    mother_points: points scored for a mother pin.
    """

    mother_exception: bool = False
    mother_points: int = 2


EGYPTIAN = Rules()
PLAKOTO = Rules(mother_exception=True)
