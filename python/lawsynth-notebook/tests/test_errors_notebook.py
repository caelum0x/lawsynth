from lawsynth_notebook.errors_notebook import ArtifactValidationError, NotebookError


def test_notebook_errors_have_a_common_base():
    assert issubclass(ArtifactValidationError, NotebookError)
