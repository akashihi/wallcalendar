import argparse
import requests
import os
import time

# Women calendar for 1987
BASE_URL = "https://www.visualhistory.ru/wp-content/uploads/2015/{}/fw87_{:02d}-{:02d}{:s}-484x700.jpg"
BASE_FILE_NAME = "{:02d}-{:02d}-{:s}.jpg"
DAY_LIMITS = {
    2: 28,
    4: 30,
    6: 30,
    9: 30,
    11: 30
}


class Downloader:
    def __init__(self, target: str, force: bool = False):
        self.target_filename = os.path.join(target, BASE_FILE_NAME)
        self.force = force

    def get_image(self, month: int, day: int, side: str):
        url_1 = BASE_URL.format("01", month, day, side)
        url_2 = BASE_URL.format("02", month, day, side)
        filename = self.target_filename.format(month, day, side)

        if not self.force:
            if os.path.isfile(filename):
                print(f"{filename} already downloaded, skipping")
                return

        r = requests.get(url_1)
        if r.status_code != 200:
            r = requests.get(url_2)
            if r.status_code != 200:
                print(f"{url_1} errored with {r.status_code}")
                return

        with open(filename, "wb") as f:
            f.write(r.content)
        print(f"Saved {filename}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("target", type=str,
                        help="Where to store downloaded images")
    parser.add_argument("-f", "--force", help="Force download of already downloaded files",
                        action="store_true")
    args = parser.parse_args()

    downloader = Downloader(args.target, args.force)
    for month in range(1, 13):
        for day in range(1, 32):
            if month in DAY_LIMITS:
                if day > DAY_LIMITS[month]:
                    continue
            downloader.get_image(month, day, "a")
            downloader.get_image(month, day, "b")
            time.sleep(1)


if __name__ == "__main__":
    main()
