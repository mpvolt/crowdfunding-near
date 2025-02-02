import { useState, useEffect, useContext } from 'react';

import { NearContext } from '@/wallets/near';
import styles from '@/styles/app.module.css';
import { HelloNearContract } from '../../config';
import { Cards } from '@/components/cards';
import { utils } from 'near-api-js';
// Contract that the app will interact with
const CONTRACT = HelloNearContract;

export default function HelloNear() {
  const { signedAccountId, wallet } = useContext(NearContext);

  const [campaignName, setCampaignName] = useState('');
  const [fundingGoal, setFundingGoal] = useState('');
  const [durationSeconds, setDurationSeconds] = useState('');
  const [allCampaigns, setAllCampaigns] = useState('loading...');
  const [loggedIn, setLoggedIn] = useState(false);
  const [showSpinner, setShowSpinner] = useState(false);

  // useEffect(() => {
  //   if (!wallet || !signedAccountId) return;
  //   wallet.viewMethod({ contractId: CONTRACT, method: 'get_all_campaigns'}).then(
  //     all_campaigns => setAllCampaigns(all_campaigns)
  //   );
  //   console.log('all_campaigns:', allCampaigns)

  // }, [wallet, signedAccountId]);

  // const newCampaign = async () => {
  //   setShowSpinner(true);

  //   try {
  //     const fundingGoalYocto = utils.format.parseNearAmount(fundingGoal.toString());

  //     const campaignId = await wallet.callMethod({
  //       contractId: CONTRACT,
  //       method: 'create_campaign',
  //       args: { name:campaignName, funding_goal: fundingGoalYocto, duration_seconds: durationSeconds },
  //       gas: THIRTY_TGAS,
  //     });
  //     console.log('Campaign created:', campaignId);
  //     const all_campaigns = await wallet.viewMethod({ contractId: CONTRACT, method: 'get_all_campaigns' });
  //     setAllCampaigns(all_campaigns);
  //   }
  //   catch (e) {
  //     console.error(e);
  //   } finally {
  //     setShowSpinner(false);
  //   }
  // };

  return (
    <main className={styles.main}>
      <div className={styles.description}>
        <p>
          Interacting with the contract: &nbsp;
          <code className={styles.code}>{CONTRACT}</code>
        </p>
      </div>

      <div className={styles.center}>
        <div className="input-group" hidden={!loggedIn}>
          <input
            type="text"
            className="form-control w-20"
            placeholder="Set Campaign Name"
            onChange={t => setCampaignName(t.target.value)}
          />
          <input
            type="text"
            className="form-control w-20"
            placeholder="Set Campaign Duration (seconds)"
            onChange={t => setDurationSeconds(t.target.value)}
          />
          <input
            type="text"
            className="form-control w-20"
            placeholder="Set Funding Goal"
            onChange={t => setFundingGoal(t.target.value)}
          />
          <div className="input-group-append">
            {/* <button className="btn btn-secondary" onClick={newCampaign}>
              <span hidden={showSpinner}> Create New Campaign </span>
              <i
                className="spinner-border spinner-border-sm"
                hidden={!showSpinner}
              ></i>
            </button> */}
          </div>
        </div>
        <div className="w-100 text-end align-text-center" hidden={loggedIn}>
          <p className="m-0"> Please login to create a campaign </p>
        </div>
      </div>
      <Cards />
    </main>
  );
}