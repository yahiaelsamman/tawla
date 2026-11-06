"""Mahbusa (Egyptian Plakoto) game engine."""
from .rules import EGYPTIAN, PLAKOTO, Rules
from .state import Move, State, format_move, legal_moves
from .game import GameResult, play_game, play_many

__all__ = ["EGYPTIAN", "PLAKOTO", "Rules", "Move", "State", "format_move", "legal_moves",
           "GameResult", "play_game", "play_many"]
