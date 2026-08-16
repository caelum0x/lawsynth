from lawsynth.trajectory import TrajectoryData


def test_trajectory_exposes_aligned_named_columns():
    trajectory = TrajectoryData((0, 1), {"x": (2, 3)})
    assert trajectory.column("x") == (2, 3)
