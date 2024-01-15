MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* These values correspond to the NRF52840 with Softdevices S140 7.0.1 */
  /* https://infocenter.nordicsemi.com/index.jsp?topic=%2Fsds_s140%2FSDS%2Fs1xx%2Fmem_usage%2Fmem_resource_reqs.html&cp=5_7_4_0_13_0_0 */
  FLASH : ORIGIN = 0x27000, LENGTH = 868K
  RAM : ORIGIN = 0x20000008, LENGTH = 0x3fff8
}
