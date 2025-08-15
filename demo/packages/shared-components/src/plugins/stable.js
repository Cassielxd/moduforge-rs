import STable, { setLicenseKey } from '@surely-vue/table'
import { encode as encodeBase64 } from 'js-base64';
import { MD5 as md5 } from 'crypto-js';
/**
 * 破解 \@surely-vue/table 授权。
 * @param  [options] 配置项。
 * @param  [options.hostname] 授权域名（默认值：`location.hostname`）。
 */
const hackLicenseKey = options => {
  const domain = options?.hostname ?? globalThis.location.hostname;
  console.log('domain', domain);
  const key = encodeBase64(
    `ORDER:00001,EXPIRY=33227712000000,DOMAIN=${domain},ULTIMATE=1,KEYVERSION=1`
  );
  const sign = md5(key).toString().toLowerCase();
  setLicenseKey(`${sign}${key}`);
};
hackLicenseKey();

// STable配置
export const setupSTable = (app) => {
  app.use(STable)
}

// 直接导出STable供组件使用
export { STable }
export default STable
