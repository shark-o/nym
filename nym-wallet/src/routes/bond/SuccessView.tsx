import React, { useContext } from 'react'
import { Box } from '@mui/system'
import { SuccessReponse, TransactionDetails } from '../../components'
import { ClientContext } from '../../context/main'

export const SuccessView: React.FC<{ details?: { amount: string; address: string } }> = ({ details }) => {
  const { userBalance } = useContext(ClientContext)
  return (
    <>
      <SuccessReponse
        title="Bonding Complete"
        subtitle="Sucessfully bonded to node with following details"
        caption={`You current balance is: ${userBalance.balance?.printable_balance}`}
      />
      {details && (
        <Box sx={{ mt: 2 }}>
          <TransactionDetails
            details={[
              { primary: 'Node', secondary: details.address },
              { primary: 'Amount', secondary: `${details.amount} punk` },
            ]}
          />
        </Box>
      )}
    </>
  )
}
