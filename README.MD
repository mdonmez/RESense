# RESense

_Reverse-engineered NitroSense GUI equivalent CLI utility for AN515-58._

---

## Why This Exists?

NitroSense’s interface stopped launching on the AN515-58, most likely after the 2026-04 Security Update (KB5083769) or the 2026-04 .NET Framework Security Update (KB5082417). Even after a full deep cleanup, uninstall, and reinstall, the issue still cannot be fixed.

This is a critical problem because it blocks access to the device’s most basic functions such as fan control and keyboard backlight control. Either Acer or Microsoft needs to release a fix for whatever these updates broke. But Acer no longer provides updates for NitroSense V3 on the Nitro 5 AN515-58, and to make things worse, they have even removed the download link for this software from the AN515-58 support page, even though it can still be downloaded from [this link](https://www.google.com/url?sa=E&q=https%3A%2F%2Fglobal-download.acer.com%2FGDFiles%2FApplication%2FNitro%2520Sense%2FNitro%2520Sense_Acer_3.01.3052_20230420_W11x64_A.zip%3Facerid%3D638688149888116955%26Step1%3D%26Step2%3D%26Step3%3DNITRO%2520AN515-58%26OS%3DALL%26LC%3Den%26BC%3DACER%26SC%3DPA_6).

Also, expecting Microsoft to fix this after breaking it on an operating system that is already this problematic is not exactly realistic either.

The purpose of this tool is to provide a working, programmable NitroSense GUI replacement after dealing with all of this mess.
