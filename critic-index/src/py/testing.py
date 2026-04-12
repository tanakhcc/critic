import math

import kraken
from PIL import Image
from kraken.rpred import rpred
from kraken import blla
import kraken.lib.models
from kraken.containers import Segmentation, BaselineLine
from kraken.lib.segmentation import calculate_polygonal_environment

with Image.open('/home/jonsch/HBCompCrit/critic/tmp/data/images/Aleppo/1_Aleppo_Codex_FULL_high-resolution-014/original.webp') as im:
    seg_model = kraken.lib.vgsl.TorchVGSLModel.load_model('/home/jonsch/HBCompCrit/critic/tmp/data/models/segmentation/SoferMahirCleanFL06Eb_83_tl_2/original.mlmodel')
    rec_model = kraken.lib.models.load_any('/home/jonsch/HBCompCrit/critic/tmp/data/models/recognition/BiblIA_01_1/original.mlmodel')
    seg = blla.segment(im, text_direction="horizontal-rl", model=seg_model)

    # seg = Segmentation(type="baselines", imagename=im, script_detection=False, lines=[BaselineLine(id="unknown", baseline=el.baseline, boundary=el.boundary) for el in seg.lines], text_direction="horizontal-rl")

    raw_bls = [el.baseline for el in seg.lines]
    pols = calculate_polygonal_environment(im, baselines=raw_bls, raise_on_error=False)
    assert len(pols) == len(raw_bls)
    new_segmentation_bls = []
    for i, pol in enumerate(pols):
        if pol is not None:
            new_segmentation_bls.append(BaselineLine(id="unknown", baseline=raw_bls[i], boundary=pol))
        else:
            bl_start, bl_end = raw_bls[i]
            bl_x = bl_end[0] - bl_start[0]
            bl_y = bl_end[1] - bl_start[1]
            bl_ortho_len = math.sqrt(bl_y**2 + bl_x**2)
            bl_normal = (bl_y / bl_ortho_len, -bl_x / bl_ortho_len)
            boundary = [
                    (bl_start[0] + bl_normal[0] * 20, bl_start[1] + bl_normal[1] * 20),
                    (bl_end[0] + bl_normal[0] * 20, bl_end[1] + bl_normal[1] *20),
                    (bl_end[0] - bl_normal[0] * 20, bl_end[1] - bl_normal[1]),
                    (bl_start[0] - bl_normal[0] * 20, bl_start[1] - bl_normal[1] * 20)
                ]
            new_segmentation_bls.append(BaselineLine(id="unknown", baseline=raw_bls[i], boundary=boundary))
    seg = Segmentation(type="baselines", imagename=im, script_detection=False, lines=new_segmentation_bls, text_direction="horizontal-rl")

    ocr = rpred(rec_model, im, seg)
    res = [el.prediction for el in ocr]
    breakpoint()
