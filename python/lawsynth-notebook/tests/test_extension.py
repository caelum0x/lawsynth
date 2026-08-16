from lawsynth_notebook.extension import extension_spec


def test_extension_is_metadata_not_an_installer():
    assert extension_spec()["requires_jupyterlab"] == ">=4"
