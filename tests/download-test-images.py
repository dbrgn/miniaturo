# Download test images from raw.pixls.us

import os
import urllib.request

DIR = os.path.dirname(os.path.realpath(__file__))

IMAGES = {
    # Fujifilm X-S10 uncompressed
    'fuji_xs10_uncompressed.raf': 'https://raw.pixls.us/getfile.php/4188/nice/Fujifilm%20-%20X-S10%20-%2014bit%20uncompressed%20(3:2).RAF',
    # Fujifilm X-S10 lossless
    'fuji_xs10_lossless.raf': 'https://raw.pixls.us/getfile.php/4189/nice/Fujifilm%20-%20X-S10%20-%2014bit%20compressed%20(3:2).RAF',
    # Canon EOS 600D
    'canon_eos600d.cr2': 'https://raw.pixls.us/getfile.php/1586/nice/Canon%20-%20EOS%20600D%20-%20RAW%20(3:2).CR2',
    # Canon EOS 5D Mark III
    'canon_5dmkiii.cr2': 'https://raw.pixls.us/getfile.php/771/nice/Canon%20-%20EOS%205D%20Mark%20III.CR2',
}

# Create testimages directory
if not os.path.exists(os.path.join(DIR, 'testimages')):
    os.makedirs(os.path.join(DIR, 'testimages'))

# Download raw files
for filename, url in IMAGES.items():
    filepath = os.path.join(DIR, 'testimages', filename)
    if os.path.exists(filepath) and os.path.isfile(filepath):
        print('FOUND    {}'.format(filename))
    else:
        print('FETCHING {}'.format(filename))
        urllib.request.urlretrieve(url, filepath)

# Write index file
with open(os.path.join(DIR, 'testimages', 'index.txt'), 'w') as f:
    for filename in IMAGES:
        f.write('{}\n'.format(filename))
print('WROTE    index.txt')
