import contextlib
import sys

from PIL import Image
from kraken.lib import vgsl
from kraken import blla
import kraken.lib.models

class DummyFile(object):
    def write(self, x): pass

@contextlib.contextmanager
def nostdout():
    save_stdout = sys.stdout
    save_stderr = sys.stderr
    sys.stdout = DummyFile()
    sys.stderr = DummyFile()
    yield
    sys.stdout = save_stdout
    sys.stderr = save_stderr


def segment(in_img, model_path, text_direction="horizontal-lr"):
    blla_model = vgsl.TorchVGSLModel.load_model(model_path)
    with Image.open(in_img) as im:
        with nostdout():
            print("this should not show");
            segmentation = blla.segment(im, text_direction=text_direction, model=blla_model)
    return segmentation

