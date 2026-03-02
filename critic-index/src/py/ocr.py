from PIL import Image
from kraken.rpred import rpred
import kraken.lib.models
from kraken.lib.segmentation import calculate_polygonal_environment
from kraken.containers import Segmentation, BaselineLine

def ocr(in_img, model_path, baselines, text_direction="horizontal-lr"):
    # load the model
    biblia = kraken.lib.models.load_any("~/.local/share/htrmopo/926633a8-35f5-5c4f-b2c2-dbb2e566e636/BiblIA_01.mlmodel")

    # we need to regain polygons before ocr
    with Image.open(in_img) as im:
        pols = calculate_polygonal_environment(im, baselines=raw_bls, raise_on_error=True)
        assert len(pols) == len(raw_bls)
        new_segmentation_bls = []
        for i, pol in enumerate(pols):
            if pols is not None:
                new_segmentation_bls.append(BaselineLine(id="unknown", baseline=raw_bls[i], boundary=pol))
        # create a new segmentation with the forced baselines
        new_segmentation = Segmentation(type="baselines", imagename=in_img, script_detection=False, lines=new_segmentation_bls, text_direction="horizontal-rl")
        res = [(el.prediction, el.baseline) for el in rpred(biblia, im, new_segmentation)]
    return res

