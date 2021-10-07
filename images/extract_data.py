import argparse
import os
import cv2
import numpy as np

BASE_FILE_NAME = "{:02d}-{:02d}-{:s}.{}"


class Processor:
    def __init__(self, source: str, target: str):
        self.source = source
        self.target = target

    def process(self, month: int, day: int):
        self.process_a_side(month, day)
        self.process_b_side(month, day)

    def process_a_side(self, month, day):
        input = os.path.join(self.source, BASE_FILE_NAME.format(month, day, "a", "jpg"))
        output_red = os.path.join(self.target, BASE_FILE_NAME.format(month, day, "a-red", "png"))
        output_black = os.path.join(self.target, BASE_FILE_NAME.format(month, day, "a-black", "png"))
        if os.path.isfile(input):
            im = self._read_crop(input)
            im = im[0:-230, :]
            im_hsv = cv2.cvtColor(im, cv2.COLOR_BGR2HSV)

            red_lower_mask = cv2.inRange(im_hsv, np.array([0, 100, 20]), np.array([25, 255, 255]))
            red_upper_mask = cv2.inRange(im_hsv, np.array([145, 100, 20]), np.array([179, 255, 255]))
            red_mask = red_lower_mask + red_upper_mask
            red_im = cv2.bitwise_and(im, im, mask=red_mask)
            red_im = cv2.cvtColor(red_im, cv2.COLOR_BGR2GRAY)
            (_, red_im) = cv2.threshold(red_im, 64, 192, cv2.THRESH_BINARY)
            red_im = cv2.copyMakeBorder(red_im, 0, 0, 38, 38, cv2.BORDER_CONSTANT, value=(0, 0, 0))
            cv2.imwrite(output_red, red_im)

            (_, im) = cv2.threshold(im, 170, 255, cv2.THRESH_BINARY)
            im = cv2.cvtColor(im, cv2.COLOR_BGR2GRAY)
            im = cv2.copyMakeBorder(im, 0, 0, 38, 38, cv2.BORDER_CONSTANT, value=(0, 0, 0))
            cv2.imwrite(output_black, im)

    def process_b_side(self, month, day):
        input = os.path.join(self.source, BASE_FILE_NAME.format(month, day, "b", "jpg"))
        output = os.path.join(self.target, BASE_FILE_NAME.format(month, day, "b", "png"))
        if os.path.isfile(input):
            im = self._read_crop(input)
            im = cv2.cvtColor(im, cv2.COLOR_BGR2GRAY)
            (_, im) = cv2.threshold(im, 170, 255, cv2.THRESH_BINARY)
            im = cv2.resize(im, (402, 648), cv2.INTER_AREA)
            im = cv2.copyMakeBorder(im, 0, 0, 39, 39, cv2.BORDER_CONSTANT, value=(255, 255, 255))
            cv2.imwrite(output, im)

    def _read_crop(self, fname):
        img = cv2.imread(fname)
        img = img[25:-25, 40:-40]
        return img

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=str,
                        help="Where downloaded images are stored")
    parser.add_argument("target", type=str,
                        help="Where to store extracted images")
    args = parser.parse_args()
    processor = Processor(args.source, args.target)
    for month in range(1, 13):
        for day in range(1, 32):
            processor.process(month, day)


if __name__ == "__main__":
    main()
