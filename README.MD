# RESense

_Reverse-engineered NitroSense GUI equivalent CLI utility for AN515-58._

---

## Why This Exists?

NitroSense’s interface stopped launching on the AN515-58, most likely after the 2026-04 Security Update (KB5083769) or the 2026-04 .NET Framework Security Update (KB5082417). Even after a full deep cleanup, uninstall, and reinstall, the issue still cannot be fixed.

This is a critical problem because it blocks access to the device’s most basic functions such as fan control and keyboard backlight control. Either Acer or Microsoft needs to release a fix for whatever these updates broke. But Acer no longer provides updates for NitroSense V3 on the Nitro 5 AN515-58, and to make things worse, they have even removed the download link for this software from the AN515-58 support page, even though it can still be downloaded from [this link](https://global-download.acer.com/GDFiles/Application/Nitro%20Sense/Nitro%20Sense_Acer_3.01.3052_20230420_W11x64_A.zip?acerid=638688149888116955&Step1=&Step2=&Step3=NITRO%20AN515-58&OS=ALL&LC=en&BC=ACER&SC=PA_6).

Also, expecting Microsoft to fix this after breaking it on an operating system that is already this problematic is not exactly realistic either.

The purpose of this tool is to provide a working, programmable NitroSense GUI replacement after dealing with all of this mess.
