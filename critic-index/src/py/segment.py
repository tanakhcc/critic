from PIL import Image
from kraken.lib import vgsl
from kraken import blla
import kraken.lib.models

def segment(in_img, model_path, text_direction="horizontal-lr"):
    blla_model = vgsl.TorchVGSLModel.load_model(model_path)
    with Image.open(in_img) as im:
        segmentation = blla.segment(im, text_direction=direction, model=blla_model)
    return segmentation

