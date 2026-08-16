from .repositories import Repository


class ProjectRepository(Repository):
    def __init__(self) -> None:
        super().__init__("project")
