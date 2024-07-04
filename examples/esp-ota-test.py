import sys
import os

idf_path = os.environ["IDF_PATH"]  # get value of IDF_PATH from environment
otatool_dir = os.path.join(idf_path, "components", "app_update")  # otatool.py lives in $IDF_PATH/components/app_update

sys.path.append(otatool_dir)  # this enables Python to find otatool module
from otatool import *  # import all names inside otatool module

target = OtatoolTarget("/dev/ttyACM0")
target.switch_ota_partition(0)
#print(target._get_otadata_info())

#target.erase_otadata()
#target.erase_ota_partition(1)
#target.switch_ota_partition(1)
