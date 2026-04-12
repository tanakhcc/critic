import contextlib
import math
import sys

from PIL import Image
from kraken.rpred import rpred
import kraken.lib.models
from kraken.lib.segmentation import calculate_polygonal_environment
from kraken.containers import Segmentation, BaselineLine

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

def ocr(in_img, model_path, baselines_and_boundaries, text_direction="horizontal-lr"):
    # load the model
    biblia = kraken.lib.models.load_any(model_path)

    # we need to regain polygons before ocr
    with Image.open(in_img) as im:
        with nostdout():
            new_segmentation_bls = []
            for baseline, boundary in baselines_and_boundaries:
                new_segmentation_bls.append(BaselineLine(id="unknown", baseline=baseline, boundary=boundary))
            # create a new segmentation with the forced baselines
            new_segmentation = Segmentation(type="baselines", imagename=in_img, script_detection=False, lines=new_segmentation_bls, text_direction=text_direction)
            res = rpred(biblia, im, new_segmentation)
            res = [(el.prediction, el.baseline) for el in res]
    return res

