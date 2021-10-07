Images and their preparation scripts
====================================

Prerequisites
=============

Scripts are made with python and python environment is managed with [Anaconda](https://www.anaconda.com/)

After getting [Anaconda](https://www.anaconda.com/) installed please issue the following commands to configure python environment:

    conda env create --file WallCalendar.yml
    conda activate WallCalendar

Handcraftet images
==================

* `moon` - moon phase icons
* `small_digits` - digits used for everything _except_ day
* `big_digits` - digits used to render a day
* `weekdays` - weekdays names
* `months` - month names

Download script
===============

`images_dl.py [-f] <target>` - will download calendar images from the [VisualHistory.ru](https://www.visualhistory.ru) 
Pay attention that depending on your location it could be a copyright violation.

`<target>` parameter set's the output directory for the images

Optional `-f` parameter will force redownloading of already downloaded images.

Preprocessing script
====================

`extract_data.py <source> <target>` - automatically converts raw calendar images into WallCalendar compatible information images.

`<source>` parameter should point to the directory containing images, downloaded with `images_dl.py`

`<target>` parameter set's the output directory for the images.

Please pay attention, that conversion quality is below average and manual editing is recommended.